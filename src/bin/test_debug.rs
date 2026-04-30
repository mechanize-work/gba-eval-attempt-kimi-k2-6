
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
    
    // Run 1 frame then trace
    emu_run_frame();
    
    eprintln!("=== After 1 frame ===");
    for i in 0..30 {
        let pc = emu_debug_pc();
        eprintln!("step {}: PC=0x{:08X}", i, pc);
        if pc < 0x0100 || pc > 0x0A000000 {
            eprintln!("SUSPICIOUS PC!");
            break;
        }
        emu_debug_step_trace();
    }
}
