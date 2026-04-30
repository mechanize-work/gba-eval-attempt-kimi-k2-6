pub struct InterruptController {
    ie: u16,
    if_: u16,
    ime: u16,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController { ie: 0, if_: 0, ime: 0 }
    }

    pub fn reset(&mut self) {
        self.ie = 0;
        self.if_ = 0;
        self.ime = 0;
    }

    pub fn irq_pending(&self) -> bool {
        self.ime != 0 && (self.ie & self.if_) != 0
    }

    pub fn request_vblank(&mut self) {
        self.if_ |= 1; // VBlank
    }

    pub fn request_vcount(&mut self) {
        self.if_ |= 1 << 2; // VCount match
    }

    pub fn read_ie(&self) -> u16 {
        self.ie
    }

    pub fn write_ie(&mut self, val: u16) {
        self.ie = val;
    }

    pub fn read_if(&self) -> u16 {
        self.if_
    }

    pub fn write_if(&mut self, val: u16) {
        self.if_ &= !val; // Writing 1 clears
    }

    pub fn read_ime(&self) -> u16 {
        self.ime
    }

    pub fn write_ime(&mut self, val: u16) {
        self.ime = val & 1;
    }
}
