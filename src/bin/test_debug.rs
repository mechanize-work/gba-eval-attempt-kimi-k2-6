
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
    
    for i in 0..55 {
        emu_run_frame();
    }
    
    eprintln!("=== Trace from frame 55 ===");
    for i in 0..55 {
        let pc = emu_debug_pc();
        if pc < 0x100 || pc > 0x0A000000 {
            eprintln!("CRASH at step {}: PC=0x{:08X}", i, pc);
            break;
        }
        emu_debug_step_trace();
    }
}
