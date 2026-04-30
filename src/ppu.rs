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
        let dots = self.cycles / 4;
        if dots >= 308 {
            self.cycles -= 308 * 4;
            
            // Render the completed scanline if visible
            if self.scanline < 160 {
                self.render_scanline(self.scanline, bus, framebuffer);
            }
            
            self.scanline += 1;
            if self.scanline == 160 {
                // VBlank start
                interrupts.request_vblank();
            }
            if self.scanline >= 228 {
                self.scanline = 0;
            }
        }
        
        // Update DISPSTAT directly on bus
        let vcount_match = (bus.dispstat >> 8) as u8;
        let mut new_dispstat = bus.dispstat & 0xFFF8;
        
        if self.cycles >= 4 * 240 {
            new_dispstat |= 2; // HBlank
        }
        
        if self.scanline >= 160 {
            new_dispstat |= 1; // VBlank
        }
        
        if self.scanline == vcount_match as u16 {
            new_dispstat |= 4; // VCount match
            if bus.dispstat & (1 << 5) != 0 {
                interrupts.request_vcount();
            }
        }
        
        bus.dispstat = new_dispstat;
        bus.vcount = self.scanline;
    }

    fn render_scanline(&mut self, ly: u16, bus: &mut crate::bus::Bus, framebuffer: &mut [u32]) {
        let mode = bus.dispcnt & 7;
        let bg0_en = bus.dispcnt & (1 << 8) != 0;
        
        if mode == 3 {
            // Bitmap mode 3: 240x160, 16-bit colors
            let row_start = ly as usize * 240;
            let vram_start = ly as usize * 240 * 2;
            for x in 0..240 {
                let addr = vram_start + x * 2;
                let color = (bus.vram[addr] as u16) | ((bus.vram[addr + 1] as u16) << 8);
                let r = ((color & 0x1F) << 3) as u32;
                let g = (((color >> 5) & 0x1F) << 3) as u32;
                let b = (((color >> 10) & 0x1F) << 3) as u32;
                framebuffer[row_start + x] = 0xFF000000 | (b << 16) | (g << 8) | r;
            }
        } else if mode == 0 {
            // Text mode 0
            if bg0_en {
                self.render_bg0_text(ly, bus, framebuffer);
            } else {
                // Fill with white or black
                let row_start = ly as usize * 240;
                for x in 0..240 {
                    framebuffer[row_start + x] = 0xFF000000;
                }
            }
        } else {
            // Other modes - just fill black for now
            let row_start = ly as usize * 240;
            for x in 0..240 {
                framebuffer[row_start + x] = 0xFF000000;
            }
        }
    }

    fn render_bg0_text(&mut self, ly: u16, bus: &crate::bus::Bus, framebuffer: &mut [u32]) {
        let row_start = ly as usize * 240;
        // BG0CNT at 0x04000008
        let bg0cnt = (bus.io[0x08] as u16) | ((bus.io[0x09] as u16) << 8);
        let screen_base = ((bg0cnt >> 8) & 0x1F) as usize * 0x800;
        let char_base = ((bg0cnt >> 2) & 3) as usize * 0x4000;
        // BG0HOFS, BG0VOFS
        let _bg0hofs = (bus.io[0x10] as u16) | ((bus.io[0x11] as u16) << 8);
        let _bg0vofs = (bus.io[0x12] as u16) | ((bus.io[0x13] as u16) << 8);
        
        // Simple render
        let mut bg_en = false;
        let bg_color = bus.dispcnt & (1 << 8) != 0;
        
        for x in 0..240 {
            if bg_color {
                framebuffer[row_start + x] = 0xFF000000;
            } else {
                framebuffer[row_start + x] = 0xFFFFFFFF;
            }
        }
    }
}
