pub struct Apu {
    sample_counter: f64,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            sample_counter: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.sample_counter = 0.0;
    }

    pub fn step(
        &mut self,
        _cycles: u32,
        _audio_buffer: &mut [i16],
        _audio_samples: &mut usize,
    ) {
        // TODO: implement actual APU
    }
}
