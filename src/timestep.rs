use std::time::{Duration, Instant};

pub struct FixedTimestep {
    timestep: Duration,
    accumulator: Duration,
    last_update: Instant,
}

impl FixedTimestep {
    pub fn new(hz: u32) -> Self {
        Self {
            timestep: Duration::from_secs_f64(1.0 / hz as f64),
            accumulator: Duration::default(),
            last_update: Instant::now(),
        }
    }
    
    pub fn update<F>(&mut self, mut fixed_update: F)
    where
        F: FnMut(),
    {
        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;
        
        self.accumulator += delta;
        
        while self.accumulator >= self.timestep {
            fixed_update();
            self.accumulator -= self.timestep;
        }
    }
}
