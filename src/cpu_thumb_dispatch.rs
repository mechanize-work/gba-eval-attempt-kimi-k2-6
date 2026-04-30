fn execute_thumb_v2(cpu: &mut Cpu, opcode: u16, bus: &mut Bus) {
    let bits15_13 = (opcode >> 13) & 7;
    let bits15_12 = (opcode >> 12) & 0xF;
    let bits15_11 = (opcode >> 11) & 0x1F;
    let bits15_10 = (opcode >> 10) & 0x3F;
    match bits15_13 {
        0b000 => {
            if bits15_11 == 0b00011 {
                // Add/subtract immediate (format 2, but actually format 00011xxx)
                // Actually format 000: bit10-9 determine
            }
            if ((opcode >> 11) & 3) == 0b11 {
                // add/subtract reg / imm
                cpu.execute_thumb_0_add_sub(opcode);
            } else {
                // move shifted register
                cpu.execute_thumb_0_shift(opcode);
            }
        }
        0b001 => cpu.execute_thumb_1(opcode),
        0b010 => {
            if bits15_11 == 0b01001 {
                // LDR Rd, [PC, #imm]
                cpu.execute_thumb_ldr_pc(opcode, bus);
            } else if bits15_10 == 0b010000 {
                // ALU register
                cpu.execute_thumb_2(opcode);
            } else if bits15_10 == 0b010001 {
                // HI register / BX
                cpu.execute_thumb_4(opcode);
            } else {
                // LDR/STR register offset, byte/hword
                cpu.execute_thumb_3(opcode, bus);
            }
        }
        0b011 => cpu.execute_thumb_3(opcode, bus), // LDR/STR immediate offset
        0b100 => {
            if bits15_12 == 0b1001 {
                // LDR/STR SP-relative (actually 1001 is SP-rel)
                cpu.execute_thumb_ldr_str_sp(opcode, bus);
            } else if bits15_12 == 0b1000 {
                // Halfword transfers (LDRH/STRH/LDRSB/LDRSH)
                cpu.execute_thumb_halfword(opcode, bus);
            } else {
                // Shouldn't happen, fallback
                cpu.execute_thumb_5(opcode, bus);
            }
        }
        0b101 => {
            if bits15_11 == 0b10100 {
                // ADD Rd, SP, #imm?
                cpu.execute_thumb_add_sp(opcode);
            } else if bits15_11 == 0b10101 {
                // ADD Rd, PC, #imm?
                cpu.execute_thumb_add_pc(opcode);
            } else {
                // PUSH / POP
                cpu.execute_thumb_6(opcode, bus);
            }
        }
        0b110 => {
            if bits15_12 == 0b1100 {
                // STMIA / LDMIA
                cpu.execute_thumb_7(opcode, bus);
            } else {
                // Conditional branch
                cpu.execute_thumb_conds(opcode);
            }
        }
        0b111 => {
            if bits15_12 == 0b1111 {
                // Long branch with link (BL)
                cpu.execute_thumb_8(opcode);
            } else if bits15_11 == 0b11100 {
                // Unconditional branch
                cpu.execute_thumb_b_uncond(opcode);
            } else if bits15_11 == 0b11101 {
                // BLX prefix or BL long branch
            } else {
                // SWI or conditional
                cpu.execute_thumb_cond_branch(opcode);
            }
        }
        _ => {}
    }
}
