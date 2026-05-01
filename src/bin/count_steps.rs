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

    for f in 0..5 {
        let mut steps = 0u64;
        let mut cycles = 0u64;
        let mut pc_hist = vec![];
        loop {
            let c = emu_debug_step_trace() as u64;
            steps += 1;
            cycles += c;
            if steps % 10000 == 0 {
                let pc = emu_debug_pc();
                pc_hist.push(pc);
            }
            if cycles >= 280896 {
                break;
            }
        }
        let pc = emu_debug_pc();
        println!("frame {} steps={} cycles={} PC=0x{:08X} hist={:?}", f, steps, cycles, pc, pc_hist);
    }
}
