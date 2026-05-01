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

    for _ in 0..20 {
        emu_debug_step_trace();
    }
    for i in 20..80 {
        let pc = emu_debug_pc();
        if i < 30 || pc >= 0x08000120 {
            emu_debug_step_trace();
        } else {
            // skip trace for speed
            emu_debug_step_trace();
        }
    }
}
