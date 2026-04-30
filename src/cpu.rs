use crate::bus::Bus;
use crate::interrupt::InterruptController;

pub struct Cpu {
    r: [u32; 16],
    cpsr: u32,
    spsr: u32,
    banked_regs: [[u32; 7]; 5],
    banked_spsr: [u32; 5],
    mode: u8,
    thumb: bool,
    halt: bool,
    pipeline: [u32; 3],
    pipeline_valid: [bool; 3],
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            r: [0; 16],
            cpsr: 0x1F,
            spsr: 0,
            banked_regs: [[0; 7]; 5],
            banked_spsr: [0; 5],
            mode: 0x1F,
            thumb: false,
            halt: false,
            pipeline: [0; 3],
            pipeline_valid: [false; 3],
        }
    }

    pub fn reset(&mut self) {
        self.r = [0; 16];
        self.cpsr = 0x1F | (1 << 7) | (1 << 6); // System mode, IRQ & FIQ disabled
        self.spsr = 0;
        self.banked_regs = [[0; 7]; 5];
        self.banked_spsr = [0; 5];
        self.mode = 0x1F;
        self.thumb = false;
        self.halt = false;
        self.pipeline = [0; 3];
        self.pipeline_valid = [false; 3];
        
        // Set PC to BIOS reset vector
        self.r[15] = 0x00000000;
        
        // Initialize pipeline
        self.pipeline[0] = 0;
        self.pipeline[1] = 0;
        self.pipeline[2] = 0;
        self.pipeline_valid = [true, true, false];
    }

    pub fn irq_disabled(&self) -> bool {
        (self.cpsr >> 7) & 1 != 0
    }

    pub fn trigger_irq(&mut self, bus: &mut Bus) {
        // Switch to IRQ mode
        let old_mode = self.mode;
        self.switch_mode(0x12);
        self.banked_spsr[(old_mode >> 0) as usize] = self.cpsr;
        self.cpsr = (self.cpsr & !0x1F) | 0x12 | (1 << 7);
        self.thumb = false;
        self.r[14] = self.r[15] + 4;
        self.r[15] = 0x00000018;
        self.pipeline_valid = [false, false, false];
    }

    fn switch_mode(&mut self, new_mode: u8) {
        // Bank register switching would go here
        self.mode = new_mode;
    }

    pub fn step(&mut self, bus: &mut Bus, interrupts: &mut InterruptController) -> u32 {
        if self.halt {
            return 1;
        }

        if self.thumb {
            self.step_thumb(bus, interrupts)
        } else {
            self.step_arm(bus, interrupts)
        }
    }

    fn step_arm(&mut self, bus: &mut Bus, _interrupts: &mut InterruptController) -> u32 {
        let pc = self.r[15] & !3;
        let opcode = bus.read32(pc);
        self.r[15] = pc + 4;

        let cond = (opcode >> 28) & 0xF;
        if !self.check_condition(cond) {
            return 1;
        }

        let inst_type = (opcode >> 26) & 0x3;
        match inst_type {
            0b00 => self.execute_data_processing(opcode, bus),
            0b01 => self.execute_load_store(opcode, bus),
            0b10 => self.execute_branch_block(opcode, bus),
            0b11 => self.execute_coprocessor(opcode, bus),
            _ => {}
        }

        self.r[15] = self.r[15] & !3;
        1
    }

    fn step_thumb(&mut self, bus: &mut Bus, _interrupts: &mut InterruptController) -> u32 {
        let pc = self.r[15] & !1;
        let opcode = bus.read16(pc);
        self.r[15] = pc + 2;
        
        let inst_type = (opcode >> 10) & 0x3F;
        match inst_type {
            _ => {}
        }

        1
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

    fn execute_data_processing(&mut self, opcode: u32, bus: &mut Bus) {
        let _opcode_type = (opcode >> 21) & 0xF;
        let s = ((opcode >> 20) & 1) != 0;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let _immediate = ((opcode >> 25) & 1) != 0;

        // Simplified: just handle basic cases
        if rd < 15 {
            self.r[rd] = self.r[rn].wrapping_add(1); // placeholder
        }

        if s && rd == 15 {
            self.cpsr = self.spsr;
        }
    }

    fn execute_load_store(&mut self, _opcode: u32, _bus: &mut Bus) {}

    fn execute_branch_block(&mut self, opcode: u32, _bus: &mut Bus) {
        let link = ((opcode >> 24) & 1) != 0;
        let offset = (opcode & 0xFFFFFF) as i32;
        let signed_offset = if offset & 0x800000 != 0 {
            offset | !0xFFFFFF
        } else {
            offset
        };

        if link {
            self.r[14] = self.r[15];
        }

        self.r[15] = ((self.r[15] as i32) + (signed_offset << 2)) as u32;
        self.pipeline_valid = [false, false, false];
    }

    fn execute_coprocessor(&mut self, _opcode: u32, _bus: &mut Bus) {}
}
