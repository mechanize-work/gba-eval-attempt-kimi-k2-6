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

    // Step through entire frame multiple times
    for frame in 0..10 {
        for step in 0..5000 {
            emu_debug_step_trace();
            // break if pc goes to BIOS
            if emu_debug_pc() >= 0x08000000 && emu_debug_pc() < 0x0D000000 {
                // in ROM space
            }
        }
        println!("--- frame {} done ---", frame);
    }
}
