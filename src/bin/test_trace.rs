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
    
    // Trace first 50 steps
    for i in 0..50 {
        let pc_before = emu_debug_pc();
        emu_debug_step_trace();
        let pc_after = emu_debug_pc();
        
        if pc_after == pc_before {
            println!("STUCK at step {} PC=0x{:08X}", i, pc_after);
            break;
        }
        
        if i > 30 && pc_after < 0x08000000 {
            println!("BIOS loop at step {} PC=0x{:08X}", i, pc_after);
            break;
        }
    }
}
