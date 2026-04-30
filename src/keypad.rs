pub struct Keypad {
    pub keys: u32,
}

impl Keypad {
    pub fn new() -> Self {
        Keypad { keys: 0 }
    }

    pub fn reset(&mut self) {
        self.keys = 0;
    }

    pub fn set_keys(&mut self, keys: u32) {
        self.keys = keys;
    }
}
