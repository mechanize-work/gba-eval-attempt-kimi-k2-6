use std::slice;

mod bus;
mod cpu;
mod ppu;
mod apu;
mod dma;
mod timer;
mod keypad;
mod interrupt;

use bus::Bus;
use cpu::Cpu;
use ppu::Ppu;
use apu::Apu;
use dma::Dma;
use timer::Timers;
use keypad::Keypad;
use interrupt::InterruptController;

const FRAME_WIDTH: usize = 240;
const FRAME_HEIGHT: usize = 160;
const AUDIO_RATE: i32 = 32768;
const AUDIO_BUFFER_SAMPLES: usize = AUDIO_RATE as usize / 60 * 4;

static mut EMULATOR: Option<Box<Emulator>> = None;

pub struct Emulator {
    pub cpu: Cpu,
    pub bus: Bus,
    pub ppu: Ppu,
    pub apu: Apu,
    pub dma: Dma,
    pub timers: Timers,
    pub keypad: Keypad,
    pub interrupts: InterruptController,
    framebuffer: Vec<u32>,
    audio_buffer: Vec<i16>,
    audio_samples: usize,
    cycles_this_frame: u64,
}

impl Emulator {
    fn new() -> Self {
        Emulator {
            cpu: Cpu::new(),
            bus: Bus::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            dma: Dma::new(),
            timers: Timers::new(),
            keypad: Keypad::new(),
            interrupts: InterruptController::new(),
            framebuffer: vec![0xFF000000; FRAME_WIDTH * FRAME_HEIGHT],
            audio_buffer: vec![0; AUDIO_BUFFER_SAMPLES * 2],
            audio_samples: 0,
            cycles_this_frame: 0,
        }
    }

    fn load_rom(&mut self, rom: &[u8]) -> bool {
        self.bus.load_rom(rom);
        self.reset();
        true
    }

    fn reset(&mut self) {
        self.cpu.reset();
        self.bus.reset();
        self.ppu.reset();
        self.apu.reset();
        self.dma.reset();
        self.timers.reset();
        self.keypad.reset();
        self.interrupts.reset();
        self.framebuffer.fill(0xFF000000);
        self.audio_buffer.fill(0);
        self.audio_samples = 0;
        self.cycles_this_frame = 0;
        self.cpu.r[15] = 0x00000000;
    }

    fn run_frame(&mut self) {
        let target_cycles = 280896;
        while self.cycles_this_frame < target_cycles {
            let dma_cycles = self.dma.step(&mut self.bus, &mut self.interrupts);
            if dma_cycles > 0 {
                self.advance_cycles(dma_cycles);
                continue;
            }
            let cycles = self.cpu.step(&mut self.bus, &mut self.interrupts);
            self.advance_cycles(cycles);
            if self.interrupts.irq_pending(&self.bus) && !self.cpu.irq_disabled() {
                self.cpu.trigger_irq(&mut self.bus);
            }
        }
        self.cycles_this_frame -= target_cycles;
    }

    fn advance_cycles(&mut self, cycles: u32) {
        self.cycles_this_frame += cycles as u64;
        self.ppu.step(cycles, &mut self.bus, &mut self.interrupts, &mut self.framebuffer);
        self.timers.step(cycles, &mut self.interrupts);
        self.apu.step(cycles, &mut self.audio_buffer, &mut self.audio_samples);
        self.dma.check_triggers(&mut self.bus, &self.ppu, &self.timers);
    }
}

#[no_mangle]
pub extern "C" fn emu_init() -> i32 {
    unsafe {
        EMULATOR = Some(Box::new(Emulator::new()));
    }
    1
}

static mut ROM_BUFFER: Vec<u8> = Vec::new();

#[no_mangle]
pub extern "C" fn emu_rom_buffer() -> *mut u8 {
    unsafe {
        ROM_BUFFER = vec![0u8; 32 * 1024 * 1024];
        ROM_BUFFER.as_mut_ptr()
    }
}

#[no_mangle]
pub extern "C" fn emu_load_rom(len: i32) -> i32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            let rom = slice::from_raw_parts(ROM_BUFFER.as_ptr(), len as usize);
            if emu.load_rom(rom) {
                return 1;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn emu_reset() -> i32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.reset();
            return 1;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn emu_set_keys(keys: u32) {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.keypad.set_keys(keys);
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_run_frame() {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.run_frame();
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_debug_pc() -> u32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.cpu.r[15]
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_debug_step_trace() -> i32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            let (cycles, _trace) = emu.cpu.step_trace(&mut emu.bus, &mut emu.interrupts);
            cycles as i32
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_framebuffer() -> *mut u32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.framebuffer.as_mut_ptr()
        } else {
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_audio_buffer() -> *mut i16 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.audio_buffer.as_mut_ptr()
        } else {
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_audio_samples() -> i32 {
    unsafe {
        if let Some(ref mut emu) = EMULATOR {
            let samples = emu.audio_samples as i32;
            emu.audio_samples = 0;
            samples
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn emu_audio_rate() -> i32 {
    AUDIO_RATE
}
