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

    for i in 0..100 {
        let pc = emu_debug_pc();
        if i < 50 || (pc >= 0x08000126 && pc <= 0x080001B0) {
            emu_debug_step_trace();
        } else {
            // fast trace
            emu_debug_step_trace();
        }
        if pc >= 0x02000000 && pc < 0x03000000 {
            eprintln!("JUMPED TO EWRAM!  Stopping.");
            break;
        }
        if pc >= 0x0A000000 {
            eprintln!("JUMPED OUT OF ROM!  Stopping.");
            break;
        }
    }
}
