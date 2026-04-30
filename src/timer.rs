pub struct Timers;

impl Timers {
    pub fn new() -> Self {
        Timers
    }

    pub fn reset(&mut self) {}

    pub fn step(
        &mut self,
        _cycles: u32,
        _interrupts: &mut crate::interrupt::InterruptController,
    ) {
    }
}
