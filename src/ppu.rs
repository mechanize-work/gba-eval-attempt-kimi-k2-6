pub struct Ppu {
    scanline: u16,
    cycles: u32,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu { scanline: 0, cycles: 0 }
    }

    pub fn reset(&mut self) {
        self.scanline = 0;
        self.cycles = 0;
    }

    pub fn step(
        &mut self,
        cycles: u32,
        bus: &mut crate::bus::Bus,
        interrupts: &mut crate::interrupt::InterruptController,
        framebuffer: &mut [u32],
    ) {
        self.cycles += cycles;
        // 4 cycles per dot, 240 dots visible + 68 dots hblank = 308 dots per scanline
        // At 16.78MHz, each dot is ~4 cycles
        let dots = self.cycles / 4;
        if dots >= 308 {
            self.cycles -= 308 * 4;
            self.scanline += 1;
            if self.scanline == 160 {
                // VBlank start
                interrupts.request_vblank();
            }
            if self.scanline >= 228 {
                self.scanline = 0;
            }
        }
        
        // Update DISPSTAT
        let dispstat = bus.read16(0x04000004);
        let vcount_match = (dispstat >> 8) as u8;
        let mut new_dispstat = dispstat & 0xFFF8;
        
        if self.cycles < 4 * 240 {
            // Visible
            if self.scanline < 160 {
                // Nothing special
            }
        } else {
            new_dispstat |= 2; // HBlank
        }
        
        if self.scanline >= 160 {
            new_dispstat |= 1; // VBlank
        }
        
        if self.scanline == vcount_match as u16 {
            new_dispstat |= 4; // VCount match
            if dispstat & (1 << 5) != 0 {
                interrupts.request_vcount();
            }
        }
        
        bus.write16(0x04000004, new_dispstat);
        bus.write16(0x04000006, self.scanline as u16);
    }
}
