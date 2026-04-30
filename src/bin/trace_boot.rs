use gba_emu::*;
use std::fs;

fn main() {
    emu_init();
    let rom_data = fs::read("dev-roms/anguna.gba").unwrap();
    let rom_buffer = emu_rom_buffer();
    unsafe {
        std::ptr::copy_nonoverlapping(rom_data.as_ptr(), rom_buffer, rom_data.len());
    }
    emu_load_rom(rom_data.len() as i32);

    for i in 0..3000 {
        let pc = emu_debug_pc();
        let in_bios = pc < 0x4000;
        let in_ewram = pc >= 0x02000000 && pc < 0x03000000;
        let in_rom = pc >= 0x08000000 && pc < 0x0A000000;
        if !in_bios && !in_ewram && !in_rom {
            eprintln!("BAD step {} PC=0x{:08X}", i, pc);
            for _ in 0..10 { emu_debug_step_trace(); }
            break;
        }
        if in_ewram || (i < 50) {
            emu_debug_step_trace();
        } else {
            // fast step
            emu_debug_step_trace();
        }
    }
}
