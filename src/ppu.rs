pub struct Ppu {
    scanline: u16,
    x: u16,
    frame: u64,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            scanline: 0,
            x: 0,
            frame: 0,
        }
    }

    pub fn reset(&mut self) {
        self.scanline = 0;
        self.x = 0;
        self.frame = 0;
    }

    pub fn step(
        &mut self,
        _cycles: u32,
        _bus: &mut crate::bus::Bus,
        _interrupts: &mut crate::interrupt::InterruptController,
        _framebuffer: &mut [u32],
    ) {
        // TODO: implement actual PPU rendering
    }
}
