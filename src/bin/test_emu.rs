use gba_emu::*;
use std::fs;

fn main() {
    println!("Testing GBA emulator...");
    
    // Initialize
    let result = emu_init();
    println!("emu_init returned: {}", result);
    
    // Load ROM
    let rom_data = fs::read("dev-roms/anguna.gba").expect("Failed to read ROM");
    println!("ROM size: {} bytes", rom_data.len());
    
    let rom_buffer = emu_rom_buffer();
    unsafe {
        std::ptr::copy_nonoverlapping(rom_data.as_ptr(), rom_buffer, rom_data.len());
    }
    
    let result = emu_load_rom(rom_data.len() as i32);
    println!("emu_load_rom returned: {}", result);
    
    // Run frames
    for i in 0..120 {
        emu_run_frame();
        let samples = emu_audio_samples();
        if i % 30 == 0 {
            println!("Frame {}, audio samples: {}", i, samples);
        }
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
