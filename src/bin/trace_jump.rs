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

    let mut history: Vec<String> = Vec::new();
    let mut saw_ewram = false;
    for i in 0..10000 {
        let pc = emu_debug_pc();
        let line = format!("step {} PC=0x{:08X}", i, pc);
        history.push(line);
        if history.len() > 30 {
            history.remove(0);
        }
        let in_ewram = pc >= 0x02000000 && pc < 0x03000000;
        if in_ewram && !saw_ewram {
            eprintln!("JUMP TO EWRAM detected!");
            for h in &history {
                eprintln!("{}", h);
            }
            saw_ewram = true;
            // trace next 20
            for _ in 0..20 {
                emu_debug_step_trace();
            }
            break;
        }
        emu_debug_step_trace();
    }
}
