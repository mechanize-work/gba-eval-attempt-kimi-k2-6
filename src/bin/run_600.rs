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
    emu_set_keys(0);
    for f in 0..600 {
        emu_run_frame();
        let pc = emu_debug_pc();
        let fb = unsafe { std::slice::from_raw_parts(emu_framebuffer(), 240 * 160) };
        let nonblack = fb.iter().filter(|&&p| p != 0xFF000000).count();
        println!("frame {} PC=0x{:08X} nonblack={}", f, pc, nonblack);
    }
}
