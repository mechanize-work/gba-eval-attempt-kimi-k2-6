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
                interrupts.request_vblank(bus);
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
                interrupts.request_vcount(bus);
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
        self.render_bg_text(ly, bus, framebuffer, 0);
    }

    fn render_bg_text(&mut self, ly: u16, bus: &crate::bus::Bus, framebuffer: &mut [u32], bg: usize) {
        let row_start = ly as usize * 240;
        
        let bgcnt_addr = 0x08 + bg * 2;
        let bgcnt = (bus.io[bgcnt_addr] as u16) | ((bus.io[bgcnt_addr + 1] as u16) << 8);
        let hofs_addr = 0x10 + bg * 4;
        let hofs = (bus.io[hofs_addr] as u16) | ((bus.io[hofs_addr + 1] as u16) << 8);
        let vofs = (bus.io[hofs_addr + 2] as u16) | ((bus.io[hofs_addr + 3] as u16) << 8);
        
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let char_base = ((bgcnt >> 2) & 3) as usize * 0x4000;
        let palette_mode = (bgcnt >> 7) & 1; // 0=16 colors, 1=256 colors
        let screen_size = (bgcnt >> 14) & 3;
        
        // Screen dimensions
        let (screen_width, screen_height) = match screen_size {
            0 => (256u16, 256u16),
            1 => (512u16, 256u16),
            2 => (256u16, 512u16),
            3 => (512u16, 512u16),
            _ => (256u16, 256u16),
        };
        
        let hofs = hofs & 0x1FF;
        let vofs = vofs & 0x1FF;
        let ly_u = ly as u16;
        
        for x in 0..240 {
            let bg_x = ((x as u16).wrapping_add(hofs)) % screen_width;
            let bg_y = (ly_u.wrapping_add(vofs)) % screen_height;
            
            let tile_x = (bg_x / 8) as usize;
            let tile_y = (bg_y / 8) as usize;
            let pixel_x = (bg_x % 8) as usize;
            let pixel_y = (bg_y % 8) as usize;
            
            // Calculate screen entry address
            let screen_entry_offset = if screen_size == 0 {
                tile_y * 32 + tile_x
            } else if screen_size == 1 {
                let block_x = tile_x / 32;
                let local_x = tile_x % 32;
                block_x * 32 * 32 + tile_y * 32 + local_x
            } else if screen_size == 2 {
                let block_y = tile_y / 32;
                let local_y = tile_y % 32;
                block_y * 32 * 32 + local_y * 32 + tile_x
            } else {
                // size 3
                let block_x = tile_x / 32;
                let block_y = tile_y / 32;
                let local_x = tile_x % 32;
                let local_y = tile_y % 32;
                block_y * 2 * 32 * 32 + block_x * 32 * 32 + local_y * 32 + local_x
            };
            
            let screen_entry_addr = screen_base + screen_entry_offset * 2;
            let screen_entry_addr_vram = (screen_entry_addr % 0x10000) as usize;
            let screen_entry_lo = bus.vram[screen_entry_addr_vram];
            let screen_entry_hi = bus.vram[(screen_entry_addr_vram + 1) % 0x10000];
            let screen_entry = (screen_entry_lo as u16) | ((screen_entry_hi as u16) << 8);
            
            let tile_num = (screen_entry & 0x3FF) as usize;
            let hflip = (screen_entry >> 10) & 1;
            let vflip = (screen_entry >> 11) & 1;
            let palette_num = ((screen_entry >> 12) & 0xF) as usize;
            
            let px = if hflip != 0 { 7 - pixel_x } else { pixel_x };
            let py = if vflip != 0 { 7 - pixel_y } else { pixel_y };
            
            let color_idx = if palette_mode == 0 {
                // 4bpp (16 colors)
                let tile_addr = char_base + tile_num * 32;
                let byte_offset = py * 4 + px / 2;
                let tile_addr_vram = (tile_addr + byte_offset) % 0x10000;
                let tile_byte = bus.vram[tile_addr_vram];
                let idx = if px % 2 == 0 { tile_byte & 0xF } else { tile_byte >> 4 };
                if idx == 0 {
                    // Transparent
                    framebuffer[row_start + x] = 0xFF000000;
                    continue;
                }
                (palette_num * 16 + idx as usize) * 2
            } else {
                // 8bpp (256 colors)
                let tile_addr = char_base + tile_num * 64;
                let byte_offset = py * 8 + px;
                let tile_addr_vram = (tile_addr + byte_offset) % 0x10000;
                let idx = bus.vram[tile_addr_vram];
                if idx == 0 {
                    framebuffer[row_start + x] = 0xFF000000;
                    continue;
                }
                idx as usize * 2
            };
            
            let pal_addr = color_idx % 0x400;
            let pal_lo = bus.palette[pal_addr];
            let pal_hi = bus.palette[(pal_addr + 1) % 0x400];
            let color = (pal_lo as u16) | ((pal_hi as u16) << 8);
            
            let r = ((color & 0x1F) << 3) as u32;
            let g = (((color >> 5) & 0x1F) << 3) as u32;
            let b = (((color >> 10) & 0x1F) << 3) as u32;
            framebuffer[row_start + x] = 0xFF000000 | (b << 16) | (g << 8) | r;
        }
    }
}
