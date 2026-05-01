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

    for i in 0..500 {
        let pc = emu_debug_pc();
        if i < 30 || (pc >= 0x08000108 && pc <= 0x08000140) || (pc >= 0x08000170 && pc <= 0x080001B0) {
            emu_debug_step_trace();
        } else {
            emu_debug_step_trace();
        }
        if pc < 0x08000000 && pc >= 0x00004000 { break; }
    }
}
