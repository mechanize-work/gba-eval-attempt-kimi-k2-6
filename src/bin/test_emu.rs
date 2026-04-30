use gba_emu::*;
use std::fs;
use std::time::Instant;

fn main() {
    emu_init();
    let rom_data = fs::read("dev-roms/anguna.gba").unwrap();
    let rom_buffer = emu_rom_buffer();
    unsafe {
        std::ptr::copy_nonoverlapping(rom_data.as_ptr(), rom_buffer, rom_data.len());
    }
    emu_load_rom(rom_data.len() as i32);
    
    let start = Instant::now();
    let mut frames = 0;
    let mut check_interval = 100;
    
    loop {
        for _ in 0..check_interval {
            emu_run_frame();
            frames += 1;
        }
        
        let fb = emu_framebuffer();
        let mut non_black = 0;
        unsafe {
            for i in 0..(240 * 160) {
                if *fb.offset(i as isize) != 0xFF000000 {
                    non_black += 1;
                }
            }
        }
        
        if non_black > 1000 {
            println!("Found {} non-black pixels at frame {} in {:?}", non_black, frames, start.elapsed());
            break;
        }
        
        if frames >= 20000 {
            println!("No display after 20000 frames in {:?}", start.elapsed());
            break;
        }
        
        if frames % 1000 == 0 {
            println!("{} frames... elapsed {:?}", frames, start.elapsed());
        }
    }
}
