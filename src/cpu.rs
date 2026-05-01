#![allow(dead_code)]
#![allow(unused_variables)]

use crate::bus::Bus;
use crate::interrupt::InterruptController;

pub struct Cpu {
    pub r: [u32; 16],
    pub cpsr: u32,
    mode: u8,
    thumb: bool,
    halt: bool,
    reg_bank: RegBank,
}

struct RegBank {
    fiq_r8_r14: [u32; 7],
    irq_r13_r14: [u32; 2],
    svc_r13_r14: [u32; 2],
    abt_r13_r14: [u32; 2],
    und_r13_r14: [u32; 2],
    spsr: [u32; 5], // fiq=0, irq=1, svc=2, abt=3, und=4
}

const MODE_USER: u32 = 0x10;
const MODE_FIQ: u32 = 0x11;
const MODE_IRQ: u32 = 0x12;
const MODE_SVC: u32 = 0x13;
const MODE_ABT: u32 = 0x17;
const MODE_UND: u32 = 0x1B;
const MODE_SYS: u32 = 0x1F;

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            r: [0; 16],
            cpsr: MODE_SYS,
            mode: MODE_SYS as u8,
            thumb: false,
            halt: false,
            reg_bank: RegBank {
                fiq_r8_r14: [0; 7], irq_r13_r14: [0; 2], svc_r13_r14: [0; 2],
                abt_r13_r14: [0; 2], und_r13_r14: [0; 2], spsr: [0; 5],
            },
        }
    }

    pub fn reset(&mut self) {
        self.r = [0; 16];
        self.cpsr = 0x1F | (1 << 7) | (1 << 6);
        self.mode = 0x1F;
        self.thumb = false;
        self.halt = false;
        self.reg_bank = RegBank {
            fiq_r8_r14: [0; 7], irq_r13_r14: [0; 2], svc_r13_r14: [0; 2],
            abt_r13_r14: [0; 2], und_r13_r14: [0; 2], spsr: [0; 5],
        };
        self.r[15] = 0x00000000;
    }

    pub fn irq_disabled(&self) -> bool {
        (self.cpsr >> 7) & 1 != 0
    }

    pub fn trigger_irq(&mut self, _bus: &mut Bus) {
        let old_mode = self.mode;
        let old_cpsr = self.cpsr;
        self.switch_mode(MODE_IRQ as u8);
        let spsr_idx = spsr_idx(old_mode);
        self.reg_bank.spsr[spsr_idx] = old_cpsr;
        self.cpsr = (self.cpsr & !0x1F) | MODE_IRQ | (1 << 7);
        self.thumb = false;
        self.r[14] = self.r[15] + 4;
        self.r[15] = 0x00000018;
    }

    fn switch_mode(&mut self, new_mode: u8) {
        let old_mode = self.mode;
        let old_spsr_idx = spsr_idx(old_mode);
        let new_spsr_idx = spsr_idx(new_mode);

        if old_mode == MODE_FIQ as u8 || new_mode == MODE_FIQ as u8 {
            // All banked regs change
            self.save_bank(old_mode);
            self.restore_bank(new_mode);
        } else if old_mode != new_mode {
            // Only R13/R14 change
            if old_mode != MODE_USER as u8 && old_mode != MODE_SYS as u8 {
                let idx = bank_idx(old_mode);
                match idx {
                    1 => { self.reg_bank.irq_r13_r14[0] = self.r[13]; self.reg_bank.irq_r13_r14[1] = self.r[14]; }
                    2 => { self.reg_bank.svc_r13_r14[0] = self.r[13]; self.reg_bank.svc_r13_r14[1] = self.r[14]; }
                    3 => { self.reg_bank.abt_r13_r14[0] = self.r[13]; self.reg_bank.abt_r13_r14[1] = self.r[14]; }
                    4 => { self.reg_bank.und_r13_r14[0] = self.r[13]; self.reg_bank.und_r13_r14[1] = self.r[14]; }
                    _ => {}
                }
            }
            if new_mode != MODE_USER as u8 && new_mode != MODE_SYS as u8 {
                let idx = bank_idx(new_mode);
                match idx {
                    1 => { self.r[13] = self.reg_bank.irq_r13_r14[0]; self.r[14] = self.reg_bank.irq_r13_r14[1]; }
                    2 => { self.r[13] = self.reg_bank.svc_r13_r14[0]; self.r[14] = self.reg_bank.svc_r13_r14[1]; }
                    3 => { self.r[13] = self.reg_bank.abt_r13_r14[0]; self.r[14] = self.reg_bank.abt_r13_r14[1]; }
                    4 => { self.r[13] = self.reg_bank.und_r13_r14[0]; self.r[14] = self.reg_bank.und_r13_r14[1]; }
                    _ => {}
                }
            }
        }
        self.mode = new_mode;
    }

    fn save_bank(&mut self, mode: u8) {
        match mode as u32 {
            MODE_FIQ => {
                for i in 0..7 { self.reg_bank.fiq_r8_r14[i] = self.r[8 + i]; }
            }
            MODE_IRQ => {
                self.reg_bank.irq_r13_r14[0] = self.r[13];
                self.reg_bank.irq_r13_r14[1] = self.r[14];
            }
            MODE_SVC => {
                self.reg_bank.svc_r13_r14[0] = self.r[13];
                self.reg_bank.svc_r13_r14[1] = self.r[14];
            }
            MODE_ABT => {
                self.reg_bank.abt_r13_r14[0] = self.r[13];
                self.reg_bank.abt_r13_r14[1] = self.r[14];
            }
            MODE_UND => {
                self.reg_bank.und_r13_r14[0] = self.r[13];
                self.reg_bank.und_r13_r14[1] = self.r[14];
            }
            _ => {}
        }
    }

    fn restore_bank(&mut self, mode: u8) {
        match mode as u32 {
            MODE_FIQ => {
                for i in 0..7 { self.r[8 + i] = self.reg_bank.fiq_r8_r14[i]; }
            }
            MODE_IRQ => {
                self.r[13] = self.reg_bank.irq_r13_r14[0];
                self.r[14] = self.reg_bank.irq_r13_r14[1];
            }
            MODE_SVC => {
                self.r[13] = self.reg_bank.svc_r13_r14[0];
                self.r[14] = self.reg_bank.svc_r13_r14[1];
            }
            MODE_ABT => {
                self.r[13] = self.reg_bank.abt_r13_r14[0];
                self.r[14] = self.reg_bank.abt_r13_r14[1];
            }
            MODE_UND => {
                self.r[13] = self.reg_bank.und_r13_r14[0];
                self.r[14] = self.reg_bank.und_r13_r14[1];
            }
            _ => {}
        }
    }

    pub fn step(&mut self, bus: &mut Bus, interrupts: &mut InterruptController) -> u32 {
        if self.halt {
            return 1;
        }

        if self.thumb {
            self.step_thumb(bus)
        } else {
            self.step_arm(bus)
        }
    }

    pub fn step_trace(&mut self, bus: &mut Bus, _interrupts: &mut InterruptController) -> (u32, String) {
        let pc = self.r[15];
        let (opcode, opcode_str) = if self.thumb {
            let op = bus.read16(pc & !1);
            (op as u32, format!("0x{:04X}", op))
        } else {
            let op = bus.read32(pc & !3);
            (op, format!("0x{:08X}", op))
        };
        
        eprint!("PC=0x{:08X} r0=0x{:08X} r1=0x{:08X} r2=0x{:08X} r3=0x{:08X} r6=0x{:08X} r12=0x{:08X} lr=0x{:08X} cpsr=0x{:08X} ", pc, self.r[0], self.r[1], self.r[2], self.r[3], self.r[6], self.r[12], self.r[14], self.cpsr);
        
        let cycles = self.step(bus, _interrupts);
        
        eprintln!("op={} -> r15=0x{:08X} thumb={}", opcode_str, self.r[15], self.thumb);
        (cycles, opcode_str)
    }

    fn step_arm(&mut self, bus: &mut Bus) -> u32 {
        let pc = self.r[15] & !3;
        let opcode = bus.read32(pc);
        // PC+8 is the architectural PC during execution of this instruction
        let pc_plus_8 = pc.wrapping_add(8);
        self.r[15] = pc + 4;

        let cond = (opcode >> 28) & 0xF;
        if !self.check_condition(cond) {
            return self.arm_cycles(opcode);
        }

        let bits2726 = (opcode >> 26) & 0x3;
        let bit25 = (opcode >> 25) & 1;

        if bits2726 == 0 {
            if (opcode >> 24) & 0xF == 0x9 && (opcode >> 4) & 1 == 1 {
                // Multiply or LDRH/STRH
                if bit25 == 0 && (opcode >> 5) & 3 == 0 && (opcode >> 22) & 1 == 0 {
                    self.execute_multiply(opcode);
                } else {
                    self.execute_halfword_transfer(opcode, bus);
                }
            } else if (opcode & 0x0FFFFFF0) == 0x012FFF10 {
                // BX - must come before MRS/MSR check!
                let rm = (opcode & 0xF) as usize;
                self.r[15] = self.r[rm] & !1;
                self.thumb = (self.r[rm] & 1) != 0;
            } else if (opcode & 0x0DB0F000) == 0x01000000 {
                self.execute_mrs(opcode);
            } else if (opcode & 0x0DB0F010) == 0x0120F000 {
                // MSR - must NOT match BX (which has bit4=1)
                self.execute_msr(opcode);
            } else {
                self.execute_data_processing(opcode, pc_plus_8);
            }
        } else if bits2726 == 1 {
            self.execute_load_store(opcode, bus);
        } else if bits2726 == 2 {
            if bit25 == 1 {
                // Branch
                self.execute_branch(opcode, pc_plus_8);
            } else {
                // Block data transfer
                self.execute_block_transfer(opcode, bus);
            }
        } else if bits2726 == 3 {
            if bit25 == 0 {
                // Coprocessor data transfer
                self.execute_coprocessor(opcode);
            } else if bit25 == 1 {
                if (opcode >> 24) & 1 == 0 {
                    // Coprocessor data operation or register transfer
                    self.execute_coprocessor(opcode);
                } else {
                    // SWI
                    self.execute_swi(opcode, bus);
                }
            }
        }

        self.r[15] = self.r[15] & !3;
        self.arm_cycles(opcode)
    }

    fn step_thumb(&mut self, bus: &mut Bus) -> u32 {
        let pc = self.r[15] & !1;
        let opcode = bus.read16(pc);
        self.r[15] = pc + 4;
        self.execute_thumb(opcode, bus);
        self.r[15] = self.r[15] & !1;
        1
    }


    fn execute_thumb(&mut self, opcode: u16, bus: &mut Bus) {
        let op = (opcode >> 13) & 7;
        match op {
            0b000 => self.execute_thumb_0(opcode),
            0b001 => self.execute_thumb_1(opcode),
            0b010 => {
                let bit12 = (opcode >> 12) & 1;
                if bit12 == 1 {
                    // 0101xxx = LDR/STR register offset
                    self.execute_thumb_3(opcode, bus);
                } else {
                    let bit11 = (opcode >> 11) & 1;
                    if bit11 == 0 {
                        let bit10 = (opcode >> 10) & 1;
                        if bit10 == 0 {
                            // 010000 = ALU register
                            self.execute_thumb_2(opcode);
                        } else {
                            // 010001 = HI register / BX
                            self.execute_thumb_4(opcode);
                        }
                    } else {
                        // 01001 = LDR PC-relative
                        let rd = ((opcode >> 8) & 7) as usize;
                        let imm = ((opcode & 0xFF) as u32) << 2;
                        // PC is instruction addr + 4, then aligned to word
                        let pc_addr = ((self.r[15] + 2) & !3);
                        let addr = pc_addr.wrapping_add(imm);
                        self.r[rd] = bus.read32(addr & !3);
                    }
                }
            }
            0b011 => {
                // LDR/STR immediate offset word/byte
                self.execute_thumb_3(opcode, bus);
            }
            0b100 => {
                let bit12 = (opcode >> 12) & 1;
                if bit12 == 0 {
                    // 1000 = halfword / sign-extended byte/halfword
                    self.execute_thumb_5(opcode, bus);
                } else {
                    // 1001 = LDR/STR SP-relative
                    self.execute_thumb_5_sp(opcode, bus);
                }
            }
            0b101 => {
                let bit11 = (opcode >> 11) & 1;
                let bit10 = (opcode >> 10) & 1;
                if bit11 == 0 {
                    // 10100 = ADD Rd, PC, #imm
                    // 10101 = ADD Rd, SP, #imm
                    let rd = ((opcode >> 8) & 7) as usize;
                    let imm = ((opcode & 0xFF) as u32) << 2;
                    if bit10 == 0 {
                        // ADD Rd, PC, #imm
                        let pc_addr = (self.r[15] + 2) & !3;
                        self.r[rd] = pc_addr.wrapping_add(imm);
                    } else {
                        // ADD Rd, SP, #imm
                        self.r[rd] = self.r[13].wrapping_add(imm);
                    }
                } else {
                    // PUSH / POP
                    self.execute_thumb_push_pop(opcode, bus);
                }
            }
            0b110 => {
                let bit12 = (opcode >> 12) & 1;
                if bit12 == 0 {
                    // 1100 = LDMIA / STMIA
                    self.execute_thumb_block(opcode, bus);
                } else {
                    // 1101 = conditional branch / SWI
                    let cond = (opcode >> 8) & 0xF;
                    if cond == 0xF {
                        // SWI
                    } else {
                        // conditional branch
                        let offset = (opcode & 0xFF) as i8;
                        if self.check_condition_thumb(cond as u32) {
                            self.r[15] = self.r[15].wrapping_add((offset as i32 * 2) as u32);
                        }
                    }
                }
            }
            0b111 => {
                let bit12 = (opcode >> 12) & 1;
                if bit12 == 1 {
                    // BL / BLX suffix
                    self.execute_thumb_bl(opcode);
                } else {
                    // unconditional branch
                    let offset = (opcode & 0x7FF) as i32;
                    let signed = if (offset >> 10) & 1 == 1 { offset | !0x7FF } else { offset };
                    self.r[15] = self.r[15].wrapping_add((signed * 2) as u32);
                }
            }
            _ => {}
        }
    }

    fn execute_thumb_0(&mut self, opcode: u16) {
        if (opcode >> 11) & 3 == 3 {
            let rd = (opcode & 7) as usize;
            let rs = ((opcode >> 3) & 7) as usize;
            let rn_offset = ((opcode >> 6) & 7) as u32;
            let i = (opcode >> 10) & 1;
            let sub = (opcode >> 9) & 1;
            let val = if i != 0 { rn_offset } else { self.r[rn_offset as usize] };
            if sub != 0 {
                self.r[rd] = self.r[rs].wrapping_sub(val);
            } else {
                self.r[rd] = self.r[rs].wrapping_add(val);
            }
        } else {
            let rd = (opcode & 7) as usize;
            let rs = ((opcode >> 3) & 7) as usize;
            let offset = ((opcode >> 6) & 0x1F) as u32;
            let shift_type = (opcode >> 11) & 3;
            let result = match shift_type {
                0b00 => if offset == 0 { self.r[rs] } else { self.r[rs] << offset },
                0b01 => if offset == 0 { 0 } else { self.r[rs] >> offset },
                0b10 => if offset == 0 { if (self.r[rs] >> 31) & 1 == 1 { 0xFFFFFFFF } else { 0 } } else { ((self.r[rs] as i32) >> offset) as u32 },
                0b11 => if offset == 0 { self.r[rs].rotate_right(1) | ((self.cpsr >> 29) & 1) << 31 } else { self.r[rs].rotate_right(offset) },
                _ => self.r[rs],
            };
            self.r[rd] = result;
        }
    }

    fn execute_thumb_1(&mut self, opcode: u16) {
        let rd = ((opcode >> 8) & 7) as usize;
        let offset = (opcode & 0xFF) as u32;
        let op_type = ((opcode >> 11) & 3) as u32;
        match op_type {
            0b00 => self.r[rd] = offset,
            0b01 => {
                let result = self.r[rd].wrapping_sub(offset);
                let carry = if self.r[rd] >= offset { 1 } else { 0 };
                let overflow = ((self.r[rd] ^ offset) & (self.r[rd] ^ result)) >> 31 == 1;
                self.update_flags(result, carry, (result >> 31) & 1, overflow);
            }
            0b10 => self.r[rd] = self.r[rd].wrapping_add(offset),
            0b11 => self.r[rd] = self.r[rd].wrapping_sub(offset),
            _ => {}
        }
    }

    fn execute_thumb_2(&mut self, opcode: u16) {
        let op = (opcode >> 6) & 0xF;
        let rs = ((opcode >> 3) & 7) as usize;
        let rd = ((opcode & 7) | ((opcode >> 4) & 8)) as usize;
        match op {
            0b0000 => self.r[rd] = self.r[rd] & self.r[rs],
            0b0001 => self.r[rd] = self.r[rd] ^ self.r[rs],
            0b0010 => { let shift = self.r[rs] & 0xFF; self.r[rd] = if shift >= 32 { 0 } else { self.r[rd] << shift }; }
            0b0011 => { let shift = self.r[rs] & 0xFF; self.r[rd] = if shift >= 32 { 0 } else { self.r[rd] >> shift }; }
            0b0100 => { let shift = self.r[rs] & 0xFF; self.r[rd] = if shift >= 32 { if (self.r[rd] >> 31) & 1 == 1 { 0xFFFFFFFF } else { 0 } } else { ((self.r[rd] as i32) >> shift) as u32 }; }
            0b0101 => { let c = (self.cpsr >> 29) & 1; self.r[rd] = self.r[rd].wrapping_add(self.r[rs]).wrapping_add(c); }
            0b0110 => { let c = (self.cpsr >> 29) & 1; self.r[rd] = self.r[rd].wrapping_sub(self.r[rs]).wrapping_sub(1 - c); }
            0b0111 => { let shift = self.r[rs] & 0xFF; self.r[rd] = if shift == 0 { self.r[rd] } else { self.r[rd].rotate_right(shift) }; }
            0b1000 => { let result = self.r[rd] & self.r[rs]; self.update_flags(result, (self.cpsr >> 29) & 1, (result >> 31) & 1, false); }
            0b1001 => self.r[rd] = 0u32.wrapping_sub(self.r[rs]),
            0b1010 => { let result = self.r[rd].wrapping_sub(self.r[rs]); let carry = if self.r[rd] >= self.r[rs] { 1 } else { 0 }; let overflow = ((self.r[rd] ^ self.r[rs]) & (self.r[rd] ^ result)) >> 31 == 1; self.update_flags(result, carry, (result >> 31) & 1, overflow); }
            0b1011 => { let result = self.r[rd].wrapping_add(self.r[rs]); let carry = if (self.r[rd] as u64 + self.r[rs] as u64) > 0xFFFFFFFF { 1 } else { 0 }; let overflow = ((self.r[rd] ^ result) & (self.r[rs] ^ result)) >> 31 == 1; self.update_flags(result, carry, (result >> 31) & 1, overflow); }
            0b1100 => self.r[rd] = self.r[rd] | self.r[rs],
            0b1101 => self.r[rd] = self.r[rd].wrapping_mul(self.r[rs]),
            0b1110 => self.r[rd] = self.r[rd] & !self.r[rs],
            0b1111 => self.r[rd] = !self.r[rs],
            _ => {}
        }
    }

    fn execute_thumb_3(&mut self, opcode: u16, bus: &mut Bus) {
        let l = (opcode >> 11) & 1;
        let b = (opcode >> 12) & 1;
        let rd = (opcode & 7) as usize;
        let rb = ((opcode >> 3) & 7) as usize;
        let offset = ((opcode >> 6) & 0x1F) as u32;
        let addr = self.r[rb].wrapping_add(if b == 1 { offset } else { offset << 2 });
        if l == 1 {
            if b == 1 { self.r[rd] = bus.read8(addr) as u32; }
            else { self.r[rd] = bus.read32(addr & !3); }
        } else {
            if b == 1 { bus.write8(addr, self.r[rd] as u8); }
            else { bus.write32(addr & !3, self.r[rd]); }
        }
    }

    fn execute_thumb_4(&mut self, opcode: u16) {
        let h1 = (opcode >> 7) & 1;
        let h2 = (opcode >> 6) & 1;
        let op = (opcode >> 8) & 3;
        let rs = (((opcode >> 3) & 7) as usize) | ((h2 as usize) << 3);
        let rd = ((opcode & 7) as usize) | ((h1 as usize) << 3);
        match op {
            0b00 => self.r[rd] = self.r[rd].wrapping_add(self.r[rs]),
            0b01 => { let result = self.r[rd].wrapping_sub(self.r[rs]); let carry = if self.r[rd] >= self.r[rs] { 1 } else { 0 }; let overflow = ((self.r[rd] ^ self.r[rs]) & (self.r[rd] ^ result)) >> 31 == 1; self.update_flags(result, carry, (result >> 31) & 1, overflow); }
            0b10 => self.r[rd] = self.r[rs],
            0b11 => { self.r[15] = self.r[rs] & !1; self.thumb = (self.r[rs] & 1) != 0; }
            _ => {}
        }
    }

    fn execute_thumb_5(&mut self, opcode: u16, bus: &mut Bus) {
        let rd = ((opcode >> 8) & 7) as usize;
        let bit12 = (opcode >> 12) & 1;
        if bit12 == 0 {
            // LDR Rd, [PC, #imm*4]
            let imm = ((opcode & 0xFF) as u32) << 2;
            let addr = (self.r[15] & !2).wrapping_add(imm);
            self.r[rd] = bus.read32(addr & !3);
        } else {
            let l = (opcode >> 11) & 1;
            let imm = ((opcode & 0xFF) as u32) << 2;
            let addr = self.r[13].wrapping_add(imm);
            if l == 0 {
                bus.write32(addr & !3, self.r[rd]);
            } else {
                self.r[rd] = bus.read32(addr & !3);
            }
        }
    }

    fn execute_thumb_6(&mut self, opcode: u16, bus: &mut Bus) {
        let l = (opcode >> 11) & 1;
        let r = (opcode >> 8) & 1;
        let rlist = opcode & 0xFF;
        if l == 0 {
            // PUSH
            let mut addr = self.r[13];
            for i in (0..8).rev() {
                if (rlist >> i) & 1 != 0 {
                    addr = addr.wrapping_sub(4);
                    bus.write32(addr & !3, self.r[i]);
                }
            }
            if r == 1 {
                addr = addr.wrapping_sub(4);
                bus.write32(addr & !3, self.r[14]);
            }
            self.r[13] = addr;
        } else {
            // POP
            let mut addr = self.r[13];
            for i in 0..8 {
                if (rlist >> i) & 1 != 0 {
                    self.r[i] = bus.read32(addr & !3);
                    addr = addr.wrapping_add(4);
                }
            }
            if r == 1 {
                let val = bus.read32(addr & !3);
                self.r[15] = val & !1;
                self.thumb = (val & 1) != 0;
                addr = addr.wrapping_add(4);
            }
            self.r[13] = addr;
        }
    }

    fn execute_thumb_7(&mut self, opcode: u16) {
        if (opcode >> 12) & 1 == 0 {
            // Conditional branch
            let cond = (opcode >> 8) & 0xF;
            let offset = (opcode & 0xFF) as i8;
            if self.check_condition_thumb(cond as u32) {
                self.r[15] = self.r[15].wrapping_add((offset as i32 * 2) as u32);
            }
        } else {
            // SWI
        }
    }

    fn execute_thumb_8(&mut self, opcode: u16) {
        if (opcode >> 11) & 1 == 0 {
            // BL prefix (low offset)
            let offset = (opcode & 0x7FF) as i32;
            let signed_off = if (offset >> 10) & 1 == 1 { offset | !0x7FF } else { offset };
            self.r[14] = self.r[15].wrapping_add((signed_off << 12) as u32);
        } else {
            // BL suffix
            self.execute_thumb_bl(opcode);
        }
    }

    fn execute_thumb_bl(&mut self, opcode: u16) {
        let offset = (opcode & 0x7FF) as i32;
        let sign = (offset >> 10) & 1;
        let signed_off = if sign == 1 {
            offset | !0x7FF
        } else {
            offset
        };
        let lr = self.r[14];
        self.r[14] = self.r[15] | 1;
        self.r[15] = ((lr & !1) as i32).wrapping_add(signed_off * 2) as u32;
    }

    fn check_condition(&self, cond: u32) -> bool {
        let n = (self.cpsr >> 31) & 1;
        let z = (self.cpsr >> 30) & 1;
        let c = (self.cpsr >> 29) & 1;
        let v = (self.cpsr >> 28) & 1;

        match cond {
            0x0 => z == 1,
            0x1 => z == 0,
            0x2 => c == 1,
            0x3 => c == 0,
            0x4 => n == 1,
            0x5 => n == 0,
            0x6 => v == 1,
            0x7 => v == 0,
            0x8 => c == 1 && z == 0,
            0x9 => c == 0 || z == 1,
            0xA => n == v,
            0xB => n != v,
            0xC => z == 0 && n == v,
            0xD => z == 1 || n != v,
            0xE => true,
            0xF => false,
            _ => true,
        }
    }

    fn check_condition_thumb(&self, cond: u32) -> bool {
        match cond {
            0x0 => ((self.cpsr >> 30) & 1) == 1, // EQ
            0x1 => ((self.cpsr >> 30) & 1) == 0, // NE
            0x2 => ((self.cpsr >> 29) & 1) == 1, // CS/HS
            0x3 => ((self.cpsr >> 29) & 1) == 0, // CC/LO
            0x4 => ((self.cpsr >> 31) & 1) == 1, // MI
            0x5 => ((self.cpsr >> 31) & 1) == 0, // PL
            0x6 => ((self.cpsr >> 28) & 1) == 1, // VS
            0x7 => ((self.cpsr >> 28) & 1) == 0, // VC
            0x8 => ((self.cpsr >> 29) & 1) == 1 && ((self.cpsr >> 30) & 1) == 0, // HI
            0x9 => ((self.cpsr >> 29) & 1) == 0 || ((self.cpsr >> 30) & 1) == 1, // LS
            0xA => ((self.cpsr >> 31) & 1) == ((self.cpsr >> 28) & 1), // GE
            0xB => ((self.cpsr >> 31) & 1) != ((self.cpsr >> 28) & 1), // LT
            0xC => ((self.cpsr >> 30) & 1) == 0 && ((self.cpsr >> 31) & 1) == ((self.cpsr >> 28) & 1), // GT
            0xD => ((self.cpsr >> 30) & 1) == 1 || ((self.cpsr >> 31) & 1) != ((self.cpsr >> 28) & 1), // LE
            0xE => true,
            0xF => false,
            _ => true,
        }
    }

    fn execute_data_processing(&mut self, opcode: u32, pc_plus_8: u32) {
        let op = (opcode >> 21) & 0xF;
        let s = ((opcode >> 20) & 1) != 0;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let i = (opcode >> 25) & 1;

        let operand2 = if i == 1 {
            self.imm_operand(opcode)
        } else {
            self.reg_operand(opcode, s, pc_plus_8)
        };

        let n = if rn == 15 { pc_plus_8 } else { self.r[rn] };
        let mut result = 0u32;
        let mut carry = (self.cpsr >> 29) & 1;
        let mut overflow = false;

        match op {
            0x0 => { // AND
                result = n & operand2.0;
                carry = operand2.1;
            }
            0x1 => { // EOR
                result = n ^ operand2.0;
                carry = operand2.1;
            }
            0x2 => { // SUB
                result = n.wrapping_sub(operand2.0);
                carry = if n >= operand2.0 { 1 } else { 0 };
                overflow = ((n ^ operand2.0) & (n ^ result)) >> 31 == 1;
            }
            0x3 => { // RSB
                result = operand2.0.wrapping_sub(n);
                carry = if operand2.0 >= n { 1 } else { 0 };
                overflow = ((operand2.0 ^ n) & (operand2.0 ^ result)) >> 31 == 1;
            }
            0x4 => { // ADD
                result = n.wrapping_add(operand2.0);
                let u_result = (n as u64) + (operand2.0 as u64);
                carry = if u_result > 0xFFFFFFFF { 1 } else { 0 };
                overflow = ((n ^ result) & (operand2.0 ^ result)) >> 31 == 1;
            }
            0x5 => { // ADC
                let c_in = (self.cpsr >> 29) & 1;
                result = n.wrapping_add(operand2.0).wrapping_add(c_in);
                carry = if (n as u64 + operand2.0 as u64 + c_in as u64) > 0xFFFFFFFF { 1 } else { 0 };
            }
            0x6 => { // SBC
                let c_in = (self.cpsr >> 29) & 1;
                result = n.wrapping_sub(operand2.0).wrapping_sub(1 - c_in);
                carry = if n >= operand2.0 + (1 - c_in) { 1 } else { 0 };
            }
            0x7 => { // RSC
                let c_in = (self.cpsr >> 29) & 1;
                result = operand2.0.wrapping_sub(n).wrapping_sub(1 - c_in);
                carry = if operand2.0 >= n + (1 - c_in) { 1 } else { 0 };
            }
            0x8 => { // TST
                result = n & operand2.0;
                carry = operand2.1;
            }
            0x9 => { // TEQ
                result = n ^ operand2.0;
                carry = operand2.1;
            }
            0xA => { // CMP
                result = n.wrapping_sub(operand2.0);
                carry = if n >= operand2.0 { 1 } else { 0 };
                overflow = ((n ^ operand2.0) & (n ^ result)) >> 31 == 1;
            }
            0xB => { // CMN
                result = n.wrapping_add(operand2.0);
                carry = if (n as u64 + operand2.0 as u64) > 0xFFFFFFFF { 1 } else { 0 };
                overflow = ((n ^ result) & (operand2.0 ^ result)) >> 31 == 1;
            }
            0xC => { // ORR
                result = n | operand2.0;
                carry = operand2.1;
            }
            0xD => { // MOV
                result = operand2.0;
                carry = operand2.1;
            }
            0xE => { // BIC
                result = n & !operand2.0;
                carry = operand2.1;
            }
            0xF => { // MVN
                result = !operand2.0;
                carry = operand2.1;
            }
            _ => {}
        }

        if op >= 8 && op <= 11 {
            // TST, TEQ, CMP, CMN - don't write result, but update flags
            if s {
                self.update_flags(result, carry, (result >> 31) & 1, overflow);
            }
        } else {
            if rd == 15 {
                if s {
                    let mode_idx = spsr_idx(self.mode);
                    let new_cpsr = self.reg_bank.spsr[mode_idx];
                    self.cpsr = new_cpsr;
                    self.thumb = ((new_cpsr >> 5) & 1) != 0;
                }
                self.r[15] = result;
            } else {
                self.r[rd] = result;
                if s {
                    self.update_flags(result, carry, (result >> 31) & 1, overflow);
                }
            }
        }
    }

    fn update_flags(&mut self, result: u32, carry: u32, n: u32, overflow: bool) {
        let z = if result == 0 { 1 } else { 0 };
        let v = if overflow { 1 } else { 0 };
        self.cpsr = (self.cpsr & !(0xF << 28)) | (n << 31) | (z << 30) | (carry << 29) | (v << 28);
    }

    fn imm_operand(&self, opcode: u32) -> (u32, u32) {
        let imm = opcode & 0xFF;
        let rotate = ((opcode >> 8) & 0xF) * 2;
        let value = imm.rotate_right(rotate);
        let carry = if rotate == 0 { (self.cpsr >> 29) & 1 } else { (value >> 31) & 1 };
        (value, carry)
    }

    fn reg_operand(&mut self, opcode: u32, _s: bool, pc_plus_8: u32) -> (u32, u32) {
        let rm = (opcode & 0xF) as usize;
        let shift_type = (opcode >> 5) & 0x3;
        let shift_amount = if ((opcode >> 4) & 1) == 0 {
            ((opcode >> 7) & 0x1F) as u32
        } else {
            let rs = ((opcode >> 8) & 0xF) as usize;
            self.r[rs] & 0xFF
        };
        let val = if rm == 15 { pc_plus_8 } else { self.r[rm] };
        let carry;
        let result;

        match shift_type {
            0b00 => { // LSL
                if shift_amount == 0 {
                    result = val;
                    carry = (self.cpsr >> 29) & 1;
                } else if shift_amount < 32 {
                    carry = (val >> (32 - shift_amount)) & 1;
                    result = val << shift_amount;
                } else if shift_amount == 32 {
                    carry = val & 1;
                    result = 0;
                } else {
                    carry = 0;
                    result = 0;
                }
            }
            0b01 => { // LSR
                if shift_amount == 0 {
                    carry = (val >> 31) & 1;
                    result = 0;
                } else if shift_amount < 32 {
                    carry = (val >> (shift_amount - 1)) & 1;
                    result = val >> shift_amount;
                } else if shift_amount == 32 {
                    carry = (val >> 31) & 1;
                    result = 0;
                } else {
                    carry = 0;
                    result = 0;
                }
            }
            0b10 => { // ASR
                if shift_amount == 0 || shift_amount >= 32 {
                    if (val >> 31) & 1 == 1 {
                        carry = 1;
                        result = 0xFFFFFFFF;
                    } else {
                        carry = 0;
                        result = 0;
                    }
                } else {
                    carry = (val >> (shift_amount - 1)) & 1;
                    result = ((val as i32) >> shift_amount) as u32;
                }
            }
            0b11 => { // ROR
                if shift_amount == 0 {
                    let c = (self.cpsr >> 29) & 1;
                    result = (c << 31) | (val >> 1);
                    carry = val & 1;
                } else {
                    result = val.rotate_right(shift_amount);
                    carry = (val >> ((shift_amount - 1) & 0x1F)) & 1;
                }
            }
            _ => { result = val; carry = 0; }
        }

        (result, carry)
    }

    fn execute_load_store(&mut self, opcode: u32, bus: &mut Bus) {
        let l = (opcode >> 20) & 1;
        let b = (opcode >> 22) & 1;
        let w = (opcode >> 21) & 1;
        let u = (opcode >> 23) & 1;
        let p = (opcode >> 24) & 1;
        let i = (opcode >> 25) & 1;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;

        let offset = if i == 1 {
            opcode & 0xFFF
        } else {
            let rm = (opcode & 0xF) as usize;
            let shift_amount = ((opcode >> 7) & 0x1F) as u32;
            let shift_type = (opcode >> 5) & 0x3;
            match shift_type {
                0b00 => self.r[rm] << shift_amount,
                0b01 => self.r[rm] >> shift_amount,
                0b10 => ((self.r[rm] as i32) >> shift_amount) as u32,
                0b11 => self.r[rm].rotate_right(shift_amount),
                _ => self.r[rm],
            }
        };

        let base = self.r[rn];
        let addr = if u == 1 { base.wrapping_add(offset) } else { base.wrapping_sub(offset) };
        let eff_addr = if p == 1 { addr } else { base };

        if l == 1 {
            // Load
            if b == 1 {
                self.r[rd] = bus.read8(eff_addr) as u32;
            } else {
                self.r[rd] = bus.read32(eff_addr & !3);
            }
            if rd == 15 {
                self.r[15] = self.r[15] & !3;
            }
        } else {
            // Store
            if b == 1 {
                bus.write8(eff_addr, (self.r[rd] & 0xFF) as u8);
            } else {
                bus.write32(eff_addr & !3, self.r[rd]);
            }
        }

        if p == 0 || w == 1 {
            self.r[rn] = addr;
        }
    }

    fn execute_branch(&mut self, opcode: u32, pc_plus_8: u32) {
        let link = ((opcode >> 24) & 1) != 0;
        let offset = (opcode & 0xFFFFFF) as i32;
        let signed_offset = if offset & 0x800000 != 0 {
            offset | !0xFFFFFF
        } else {
            offset
        };

        if link {
            self.r[14] = pc_plus_8.wrapping_sub(4);
        }

        self.r[15] = ((pc_plus_8 as i32).wrapping_add(signed_offset << 2)) as u32;
    }

    fn execute_mrs(&mut self, opcode: u32) {
        let rd = ((opcode >> 12) & 0xF) as usize;
        let r = (opcode >> 22) & 1;
        let val = if r == 1 {
            // SPSR
            let idx = spsr_idx(self.mode);
            self.reg_bank.spsr[idx]
        } else {
            self.cpsr
        };
        if rd < 15 {
            self.r[rd] = val;
        }
    }

    fn execute_msr(&mut self, opcode: u32) {
        let i = (opcode >> 25) & 1;
        let r = (opcode >> 22) & 1;
        let field_mask = (opcode >> 16) & 0xF;

        let val = if i == 1 {
            let imm = opcode & 0xFF;
            let rotate = ((opcode >> 8) & 0xF) * 2;
            imm.rotate_right(rotate)
        } else {
            let rm = (opcode & 0xF) as usize;
            self.r[rm]
        };

        if r == 0 {
            let mut mask = 0u32;
            if field_mask & 1 != 0 { mask |= 0x000000FF; }
            if field_mask & 2 != 0 { mask |= 0x0000FF00; }
            if field_mask & 4 != 0 { mask |= 0x00FF0000; }
            if field_mask & 8 != 0 { mask |= 0xFF000000; }
            self.cpsr = (self.cpsr & !mask) | (val & mask);
            
            // Switch mode if control bits were modified
            if field_mask & 1 != 0 {
                let new_mode = (self.cpsr & 0x1F) as u8;
                if new_mode != self.mode {
                    self.switch_mode(new_mode);
                }
            }
        }
    }

    fn execute_multiply(&mut self, opcode: u32) {
        let rd = ((opcode >> 16) & 0xF) as usize;
        let rn = ((opcode >> 12) & 0xF) as usize;
        let rs = ((opcode >> 8) & 0xF) as usize;
        let rm = (opcode & 0xF) as usize;
        let s = (opcode >> 20) & 1;
        let a = (opcode >> 21) & 1;
        let u = (opcode >> 22) & 1;

        if u == 0 {
            // MUL / MLA
            let mut result = self.r[rm].wrapping_mul(self.r[rs]);
            if a == 1 {
                result = result.wrapping_add(self.r[rn]);
            }
            self.r[rd] = result & 0xFFFFFFFF;
            if s == 1 {
                let n = (result >> 31) & 1;
                let z = if result == 0 { 1 } else { 0 };
                self.cpsr = (self.cpsr & !((1 << 31) | (1 << 30) | (1 << 29))) | (n << 31) | (z << 30);
            }
        }
    }

    fn execute_halfword_transfer(&mut self, opcode: u32, _bus: &mut Bus) {
        // Simplified
    }

    fn execute_block_transfer(&mut self, opcode: u32, bus: &mut Bus) {
        let pre = (opcode >> 24) & 1;
        let up = (opcode >> 23) & 1;
        let s = (opcode >> 22) & 1;
        let w = (opcode >> 21) & 1;
        let l = (opcode >> 20) & 1;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rlist = opcode & 0xFFFF;

        let start_addr = self.r[rn];
        let mut addr = start_addr;

        let count = rlist.count_ones();
        if count == 0 {
            if l == 1 {
                self.r[15] = bus.read32(start_addr & !3);
                self.r[15] = self.r[15] & !3;
            } else {
                bus.write32(start_addr & !3, self.r[15] + 4);
            }
            if w == 1 {
                self.r[rn] = start_addr.wrapping_add(if up == 1 { 64 } else { (0u32).wrapping_sub(64) });
            }
            return;
        }

        let offset = count * 4;
        let first_addr = if up == 1 { start_addr } else { start_addr.wrapping_sub(offset) };

        if l == 1 {
            // Load
            for i in 0..16 {
                if (rlist >> i) & 1 != 0 {
                    addr = if up == 1 { start_addr + i * 4 } else { start_addr - offset + i * 4 };
                    let eff_addr = if pre == 1 { addr + 4 } else { addr };
                    self.r[i as usize] = bus.read32(eff_addr & !3);
                }
            }
            if (rlist & (1 << 15)) != 0 {
                self.r[15] = self.r[15] & !3;
                self.thumb = false;
                if s == 1 {
                    let mode_idx = spsr_idx(self.mode);
                    let new_cpsr = self.reg_bank.spsr[mode_idx];
                    self.cpsr = new_cpsr;
                    self.thumb = ((new_cpsr >> 5) & 1) != 0;
                }
            }
        } else {
            // Store
            for i in 0..16 {
                if (rlist >> i) & 1 != 0 {
                    addr = if up == 1 { start_addr + i * 4 } else { start_addr - offset + i * 4 };
                    let eff_addr = if pre == 1 { addr + 4 } else { addr };
                    bus.write32(eff_addr & !3, self.r[i as usize]);
                }
            }
        }

        if w == 1 {
            if up == 1 {
                self.r[rn] = self.r[rn].wrapping_add(offset);
            } else {
                self.r[rn] = self.r[rn].wrapping_sub(offset);
            }
        }
    }

    fn execute_swi(&mut self, _opcode: u32, _bus: &mut Bus) {
        // BIOS SWI handling - just skip
    }

    fn execute_coprocessor(&mut self, _opcode: u32) {
        // Ignored
    }

    fn arm_cycles(&self, opcode: u32) -> u32 {
        let bits2526 = (opcode >> 25) & 0x3;
        match bits2526 {
            0b00 => {
                if (opcode >> 24) & 0xF == 0x9 && (opcode >> 4) & 1 == 1 {
                    let s = (opcode >> 22) & 1;
                    if s == 0 { // Multiply
                        let m = (opcode >> 4) & 1;
                        let rs = ((opcode >> 8) & 0xF) as usize;
                        let bits = self.r[rs].count_ones();
                        let base = if m == 0 { 1 } else { 2 };
                        let s_iter = if bits % 2 == 0 { 1 } else { 0 };
                        (base + s_iter + (bits / 2)) as u32
                    } else {
                        1
                    }
                } else {
                    1
                }
            }
            0b01 => {
                // LDR/STR
                let rd = ((opcode >> 12) & 0xF) as usize;
                let i = (opcode >> 25) & 1;
                if i == 0 { 1 } else { 2 }
            }
            0b10 => {
                if (opcode >> 24) & 1 == 1 { // Branch
                    3
                } else {
                    // Block transfer
                    let n = (opcode & 0xFFFF).count_ones();
                    n.saturating_sub(1) + n
                }
            }
            _ => 1,
        }
    }
}

fn spsr_idx(mode: u8) -> usize {
    match mode as u32 {
        MODE_FIQ => 0,
        MODE_IRQ => 1,
        MODE_SVC => 2,
        MODE_ABT => 3,
        MODE_UND => 4,
        _ => 0,
    }
}

fn bank_idx(mode: u8) -> usize {
    match mode as u32 {
        MODE_IRQ => 0,
        MODE_SVC => 1,
        MODE_ABT => 2,
        MODE_UND => 3,
        _ => 0,
    }
}
