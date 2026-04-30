pub struct Bus {
    bios: [u8; 0x4000],
    ewram: [u8; 0x40000],
    iwram: [u8; 0x8000],
    rom: Vec<u8>,
    io: [u8; 0x400],
    palette: [u8; 0x400],
    vram: [u8; 0x18000],
    oam: [u8; 0x400],
}

impl Bus {
    pub fn new() -> Self {
        let mut bios = [0u8; 0x4000];
        // Load BIOS stub
        if let Ok(data) = std::fs::read("spec/gba_bios_stub.bin") {
            let len = data.len().min(0x4000);
            bios[..len].copy_from_slice(&data[..len]);
        }
        
        Bus {
            bios,
            ewram: [0; 0x40000],
            iwram: [0; 0x8000],
            rom: vec![0; 32 * 1024 * 1024],
            io: [0; 0x400],
            palette: [0; 0x400],
            vram: [0; 0x18000],
            oam: [0; 0x400],
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
    }

    pub fn read8(&self, addr: u32) -> u8 {
        match addr & 0x0F000000 {
            0x00000000 => self.bios[(addr & 0x3FFF) as usize],
            0x02000000 => self.ewram[(addr & 0x3FFFF) as usize],
            0x03000000 => self.iwram[(addr & 0x7FFF) as usize],
            0x04000000 => self.io[(addr & 0x3FF) as usize],
            0x05000000 => self.palette[(addr & 0x3FF) as usize],
            0x06000000 => self.vram[(addr & 0x1FFFF) as usize % 0x18000],
            0x07000000 => self.oam[(addr & 0x3FF) as usize],
            0x08000000 | 0x09000000 | 0x0A000000 | 0x0B000000 | 0x0C000000 | 0x0D000000 => {
                self.rom[(addr & 0x1FFFFFF) as usize % self.rom.len()]
            }
            0x0E000000 | 0x0F000000 => 0,
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
        match addr & 0x0F000000 {
            0x00000000 => {},
            0x02000000 => self.ewram[(addr & 0x3FFFF) as usize] = val,
            0x03000000 => self.iwram[(addr & 0x7FFF) as usize] = val,
            0x04000000 => self.io_write8(addr, val),
            0x05000000 => self.palette[(addr & 0x3FF) as usize] = val,
            0x06000000 => self.vram[(addr & 0x1FFFF) as usize % 0x18000] = val,
            0x07000000 => self.oam[(addr & 0x3FF) as usize] = val,
            0x08000000..=0x0D000000 => {},
            0x0E000000 | 0x0F000000 => {},
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

    fn io_write8(&mut self, addr: u32, val: u8) {
        self.io[(addr & 0x3FF) as usize] = val;
    }
}
