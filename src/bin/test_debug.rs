
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
    
    // Run 120 frames
    for i in 0..120 {
        emu_run_frame();
    }
    
    // Read framebuffer
    let fb_ptr = emu_framebuffer();
    let fb = unsafe { std::slice::from_raw_parts(fb_ptr, 240 * 160) };
    
    let black = fb.iter().filter(|&&p| p == 0xFF000000).count();
    let white = fb.iter().filter(|&&p| p == 0xFFFFFFFF).count();
    let other = fb.len() - black - white;
    eprintln!("Frame 120: black={}, white={}, other={}", black, white, other);
    
    // Show some pixels
    for y in [0, 80, 159] {
        let row_start = y * 240;
        let mut line = String::new();
        for x in [0, 60, 120, 180, 239] {
            let p = fb[row_start + x];
            line.push_str(&format!("({},{})=0x{:08X} ", x, y, p));
        }
        eprintln!("{}", line);
    }
}
