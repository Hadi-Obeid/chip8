use sdl2::audio::AudioCallback;
use std::f32::consts::PI;

pub struct SquareWave {
    pub phase_inc: f32,
    pub phase: f32,
    pub volume: f32
}
struct SineWave {
    freq: f32,
    oscillator: Oscillator,
}

impl SineWave {
    pub fn new(freq: f32, volume: f32) -> Self {
        SineWave {
            freq,
            oscillator: Oscillator::new(freq, volume)
        }
    }
}

struct Oscillator {
    current_step: f32,
    step_size: f32,
    volume: f32,
}

impl Oscillator {
    pub fn new(rate: f32, volume: f32) -> Self {
        Oscillator { current_step: 0., step_size: (2. * PI) / rate, volume }
    }

    pub fn next(&mut self) -> f32 {
        self.current_step += self.step_size;
        return self.current_step.sin() * self.volume;
    }
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
impl AudioCallback for SineWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a sine wave
        for x in out.iter_mut() {
            *x = self.oscillator.next();
        }
    }
}