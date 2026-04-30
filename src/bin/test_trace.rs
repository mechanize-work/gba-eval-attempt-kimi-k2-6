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

    println!("Running 100000 CPU cycles...");
    
    // We need to manually step through a few cycles
    // But our API only supports running frames...
    // Let's just run one frame and check PC
    
    for frame in 0..5 {
        let pc_before = emu_debug_pc();
        emu_run_frame();
        let pc_after = emu_debug_pc();
        println!("Frame {}: PC before=0x{:08X}, after=0x{:08X}", frame, pc_before, pc_after);
    }
}
