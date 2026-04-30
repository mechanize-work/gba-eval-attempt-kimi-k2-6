pub struct InterruptController;

impl InterruptController {
    pub fn new() -> Self {
        InterruptController
    }

    pub fn reset(&mut self) {}

    pub fn irq_pending(&self, bus: &crate::bus::Bus) -> bool {
        let ie = bus.read_ie();
        let if_ = bus.read_if();
        let ime = bus.read_ime();
        ime != 0 && (ie & if_) != 0
    }

    pub fn request_vblank(&self, bus: &mut crate::bus::Bus) {
        let if_ = bus.read_if();
        bus.write_if(if_ | 1);
    }

    pub fn request_vcount(&self, bus: &mut crate::bus::Bus) {
        let if_ = bus.read_if();
        bus.write_if(if_ | (1 << 2));
    }
}
