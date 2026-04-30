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
    
    for frame in 0..600 {
        emu_run_frame();
    }
    
    // Check framebuffer
    let fb = emu_framebuffer();
    let mut black = 0;
    let mut white = 0;
    let mut other = 0;
    unsafe {
        for i in 0..(240 * 160) {
            let p = *fb.offset(i as isize);
            if p == 0xFF000000 { black += 1; }
            else if p == 0xFFFFFFFF { white += 1; }
            else { other += 1; }
        }
    }
    println!("After 600 frames: black={}, white={}, other={}", black, white, other);
    
    // Run for another 600
    for frame in 0..600 {
        emu_run_frame();
    }
    
    let mut black = 0;
    let mut white = 0;
    let mut other = 0;
    unsafe {
        for i in 0..(240 * 160) {
            let p = *fb.offset(i as isize);
            if p == 0xFF000000 { black += 1; }
            else if p == 0xFFFFFFFF { white += 1; }
            else { other += 1; }
        }
    }
    println!("After 1200 frames: black={}, white={}, other={}", black, white, other);
}
