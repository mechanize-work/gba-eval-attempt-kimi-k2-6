use gba_emu::*;
use std::fs;

fn main() {
    println!("Testing GBA emulator trace...");
    
    emu_init();
    let rom_data = fs::read("dev-roms/anguna.gba").unwrap();
    let rom_buffer = emu_rom_buffer();
    unsafe {
        std::ptr::copy_nonoverlapping(rom_data.as_ptr(), rom_buffer, rom_data.len());
    }
    emu_load_rom(rom_data.len() as i32);

    // Trace the first 500 instructions
    for i in 0..500 {
        let result = emu_debug_step_trace();
        if result == 0 {
            println!("CPU halted or error at step {}", i);
            break;
        }
    }
}
