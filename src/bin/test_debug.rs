
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
    
    // Press Start
    emu_set_keys(0x008);
    
    for i in 0..120 {
        emu_run_frame();
        if i == 5 { emu_set_keys(0x000); }
    }
    
    let fb_ptr = emu_framebuffer();
    let fb = unsafe { std::slice::from_raw_parts(fb_ptr, 240 * 160) };
    let black = fb.iter().filter(|&&p| p == 0xFF000000).count();
    let white = fb.iter().filter(|&&p| p == 0xFFFFFFFF).count();
    let other = fb.len() - black - white;
    eprintln!("After 120 frames with Start: black={}, white={}, other={}", black, white, other);
}
