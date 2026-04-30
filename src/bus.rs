pub struct Bus {
    pub bios: [u8; 0x4000],
    pub ewram: [u8; 0x40000],
    pub iwram: [u8; 0x8000],
    pub io: [u8; 0x400],
    pub palette: [u8; 0x400],
    pub vram: [u8; 0x18000],
    pub oam: [u8; 0x400],
    pub rom: Vec<u8>,

    // IO registers
    pub dispcnt: u16,
    pub dispstat: u16,
    pub vcount: u16,
    pub keycnt: u16,
}

impl Bus {
    pub fn new() -> Self {
        let bios_data = include_bytes!("../spec/gba_bios_stub.bin");
        let mut bios = [0u8; 0x4000];
        let len = bios_data.len().min(0x4000);
        bios[..len].copy_from_slice(&bios_data[..len]);
        
        Bus {
            bios,
            ewram: [0; 0x40000],
            iwram: [0; 0x8000],
            io: [0; 0x400],
            palette: [0; 0x400],
            vram: [0; 0x18000],
            oam: [0; 0x400],
            rom: vec![0; 32 * 1024 * 1024],
            dispcnt: 0,
            dispstat: 0,
            vcount: 0,
            keycnt: 0,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let len = rom.len().min(self.rom.len());
        self.rom[..len].copy_from_slice(&rom[..len]);
    }

    pub fn reset(&mut self) {
        self.ewram.fill(0);
        self.iwram.fill(0);
        self.io.fill(0);
        self.palette.fill(0);
        self.vram.fill(0);
        self.oam.fill(0);
        self.dispcnt = 0;
        self.dispstat = 0;
        self.vcount = 0;
        self.keycnt = 0;
    }

    pub fn read8(&self, addr: u32) -> u8 {
        let region = addr & 0x0F000000;
        let offset = addr & 0x00FFFFFF;
        match region {
            0x00000000 => self.bios[(offset & 0x3FFF) as usize],
            0x02000000 => self.ewram[(offset & 0x3FFFF) as usize],
            0x03000000 => self.iwram[(offset & 0x7FFF) as usize],
            0x04000000 => self.io_read8(offset & 0x3FF),
            0x05000000 => self.palette[(offset & 0x3FF) as usize],
            0x06000000 => self.vram_read(offset),
            0x07000000 => self.oam[(offset & 0x3FF) as usize],
            0x08000000 | 0x09000000 | 0x0A000000 | 0x0B000000 | 0x0C000000 | 0x0D000000 => {
                self.rom[((offset & 0x1FFFFFF) as usize) % self.rom.len()]
            }
            _ => 0,
        }
    }

    pub fn read16(&self, addr: u32) -> u16 {
        let a = addr & !1;
        u16::from_le_bytes([self.read8(a), self.read8(a + 1)])
    }

    pub fn read32(&self, addr: u32) -> u32 {
        let a = addr & !3;
        u32::from_le_bytes([
            self.read8(a),
            self.read8(a + 1),
            self.read8(a + 2),
            self.read8(a + 3),
        ])
    }

    pub fn write8(&mut self, addr: u32, val: u8) {
        let region = addr & 0x0F000000;
        let offset = addr & 0x00FFFFFF;
        match region {
            0x00000000 => {},
            0x02000000 => self.ewram[(offset & 0x3FFFF) as usize] = val,
            0x03000000 => self.iwram[(offset & 0x7FFF) as usize] = val,
            0x04000000 => self.io_write8(offset & 0x3FF, val),
            0x05000000 => self.palette[(offset & 0x3FF) as usize] = val,
            0x06000000 => self.vram_write(offset, val),
            0x07000000 => self.oam[(offset & 0x3FF) as usize] = val,
            _ => {},
        }
    }

    pub fn write16(&mut self, addr: u32, val: u16) {
        let a = addr & !1;
        self.write8(a, val as u8);
        self.write8(a + 1, (val >> 8) as u8);
    }

    pub fn write32(&mut self, addr: u32, val: u32) {
        let a = addr & !3;
        self.write8(a, val as u8);
        self.write8(a + 1, (val >> 8) as u8);
        self.write8(a + 2, (val >> 16) as u8);
        self.write8(a + 3, (val >> 24) as u8);
    }

    fn vram_read(&self, offset: u32) -> u8 {
        let addr = offset & 0x1FFFF;
        if addr >= 0x18000 {
            self.vram[(addr - 0x8000) as usize]
        } else {
            self.vram[addr as usize]
        }
    }

    fn vram_write(&mut self, offset: u32, val: u8) {
        let addr = offset & 0x1FFFF;
        if addr >= 0x18000 {
            self.vram[(addr - 0x8000) as usize] = val;
        } else {
            self.vram[addr as usize] = val;
        }
    }

    fn io_read8(&self, addr: u32) -> u8 {
        let offset = addr as usize;
        match addr {
            0x00 => (self.dispcnt & 0xFF) as u8,
            0x01 => ((self.dispcnt >> 8) & 0xFF) as u8,
            0x04 => (self.dispstat & 0xFF) as u8,
            0x05 => ((self.dispstat >> 8) & 0xFF) as u8,
            0x06 => (self.vcount & 0xFF) as u8,
            0x07 => ((self.vcount >> 8) & 0xFF) as u8,
            _ => self.io[offset],
        }
    }

    fn io_write8(&mut self, addr: u32, val: u8) {
        let offset = addr as usize;
        match addr {
            0x00 => self.dispcnt = (self.dispcnt & 0xFF00) | (val as u16),
            0x01 => self.dispcnt = (self.dispcnt & 0x00FF) | ((val as u16) << 8),
            0x04 => self.dispstat = (self.dispstat & 0xFF00) | (val as u16),
            0x05 => self.dispstat = (self.dispstat & 0x00FF) | ((val as u16) << 8),
            _ => self.io[offset] = val,
        }
    }
}
