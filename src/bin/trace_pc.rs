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

    for f in 0..120 {
        emu_run_frame();
        let pc = emu_debug_pc();
        if pc < 0x08000000 || pc >= 0x0A000000 {
            eprintln!("frame {} PC=0x{:08X}", f, pc);
            // trace next 20 steps
            for s in 0..20 {
                let pc2 = emu_debug_pc();
                eprintln!("  step {} PC=0x{:08X}", s, pc2);
                emu_debug_step_trace();
            }
            break;
        }
    }
}
