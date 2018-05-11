use prelude::*;

#[derive(Clone)]
pub struct Timer {
    start: Instant,
    pause_start: Option<Instant>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start: Instant::now(),
            pause_start: None,
        }
    }
    pub fn pause(self: &mut Self) {
        if self.pause_start == None {
            self.pause_start = Some(Instant::now());
        }
    }
    pub fn resume(self: &mut Self) {
        if let Some(pause_start) = self.pause_start {
            let paused_for = Instant::now() - pause_start;
            self.start += paused_for;
            self.pause_start = None;
        }
    }
    pub fn elapsed(self: &Self) -> Duration {
        if let Some(pause_start) = self.pause_start {
            pause_start - self.start 
        } else {
            Instant::now() - self.start
        }
    }
    pub fn elapsed_f32(self: &Self) -> f32 {
        let elapsed = self.elapsed();
        elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f64 / 1000000000.0) as f32
    }
}