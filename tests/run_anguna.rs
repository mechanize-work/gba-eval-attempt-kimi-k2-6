use std::fs;

#[test]
fn test_run_anguna() {
    let rom = fs::read("/task/dev-roms/anguna.gba").expect("rom missing");
    let _emu = gba_emu::emu_init();
    let buf = gba_emu::emu_rom_buffer();
    unsafe { std::ptr::copy_nonoverlapping(rom.as_ptr(), buf, rom.len()) };
    gba_emu::emu_load_rom(rom.len() as i32);
    gba_emu::emu_set_keys(0);
    for f in 0..10 {
        gba_emu::emu_run_frame();
        let fb = unsafe { std::slice::from_raw_parts(gba_emu::emu_framebuffer(), 240 * 160) };
        let mut nonblack = 0;
        let mut counts = std::collections::HashMap::new();
        for pixel in fb {
            if *pixel != 0 {
                nonblack += 1;
            }
            *counts.entry(*pixel).or_insert(0) += 1;
        }
        println!("frame {} nonblack: {}", f, nonblack);
        let mut top: Vec<_> = counts.iter().collect();
        top.sort_by(|a,b| b.1.cmp(a.1));
        for (col, cnt) in top.iter().take(5) {
            println!("  0x{:08X} : {}", col, cnt);
        }
        std::fs::write(format!("/tmp/emu_fb_{}.bin", f), unsafe {
            std::slice::from_raw_parts(fb.as_ptr() as *const u8, fb.len()*4)
        }).unwrap();
    }
}
