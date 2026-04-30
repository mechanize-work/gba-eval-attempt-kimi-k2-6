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
    
    // Trace first instruction
    for _ in 0..5 {
        emu_debug_step_trace();
    }
    println!("\nRunning 120 frames...");
    for i in 0..120 {
        emu_run_frame();
    }
    
    // Check framebuffer
    let fb = emu_framebuffer();
    let mut non_black = 0;
    let mut non_white = 0;
    unsafe {
        for i in 0..(240 * 160) {
            let pixel = *fb.offset(i as isize);
            if pixel != 0xFF000000 {
                non_black += 1;
            }
            if pixel != 0xFFFFFFFF {
                non_white += 1;
            }
        }
    }
    println!("Non-black pixels: {}", non_black);
    println!("Non-white pixels: {}", non_white);
    println!("Done!");
}
