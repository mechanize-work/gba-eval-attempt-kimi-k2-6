pub struct Dma;

impl Dma {
    pub fn new() -> Self {
        Dma
    }

    pub fn reset(&mut self) {}

    pub fn step(
        &mut self,
        _bus: &mut crate::bus::Bus,
        _interrupts: &mut crate::interrupt::InterruptController,
    ) -> u32 {
        0
    }

    pub fn check_triggers(
        &mut self,
        _bus: &mut crate::bus::Bus,
        _ppu: &crate::ppu::Ppu,
        _timers: &crate::timer::Timers,
    ) {}
}
