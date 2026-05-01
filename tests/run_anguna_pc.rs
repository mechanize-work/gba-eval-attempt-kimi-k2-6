use std::fs;

#[test]
fn test_run_anguna_pc() {
    let rom = fs::read("/task/dev-roms/anguna.gba").expect("rom missing");
    gba_emu::emu_init();
    let buf = gba_emu::emu_rom_buffer();
    unsafe { std::ptr::copy_nonoverlapping(rom.as_ptr(), buf, rom.len()) };
    gba_emu::emu_load_rom(rom.len() as i32);
    gba_emu::emu_set_keys(0);
    for f in 0..120 {
        gba_emu::emu_run_frame();
        let pc = gba_emu::emu_debug_pc();
        let fb = unsafe { std::slice::from_raw_parts(gba_emu::emu_framebuffer(), 240 * 160) };
        let nonblack = fb.iter().filter(|&&p| p != 0xFF000000).count();
        println!("frame {} PC=0x{:08X} nonblack={}", f, pc, nonblack);
        if nonblack > 0 { break; }
    }
}
