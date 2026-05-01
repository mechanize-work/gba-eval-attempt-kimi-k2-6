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
    for f in 0..120 {
        emu_run_frame();
        let pc = emu_debug_pc();
        let in_rom = pc >= 0x08000000 && pc < 0x0A000000;
        let in_bios = pc < 0x4000;
        let in_ewram = pc >= 0x02000000 && pc < 0x03000000;
        println!("frame {} PC=0x{:08X} (rom={} bios={} ewram={})", f, pc, in_rom, in_bios, in_ewram);
    }
}
