pub struct InterruptController;

impl InterruptController {
    pub fn new() -> Self {
        InterruptController
    }

    pub fn reset(&mut self) {}

    pub fn irq_pending(&self) -> bool {
        false
    }
}
