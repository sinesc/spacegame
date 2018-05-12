use prelude::*;

#[derive(Clone)]
pub struct Timeframe {
    total: Duration,
    factor_start: Instant,
    factor_value: f32,
}

impl Timeframe {
    /**
     * Creates a new timeframe.
     */
    pub fn new() -> Timeframe {
        Timeframe {
            total: Duration::new(0, 0),
            factor_start: Instant::now(),
            factor_value: 1.0,
        }
    }

    /**
     * Returns the elapsed duration of time since the timeframe began.
     */
    pub fn elapsed(self: &Self) -> Duration {
        if self.factor_value == 1.0 {
            self.total + (Instant::now() - self.factor_start)
        } else if self.factor_value == 0.0 {
            self.total
        } else {
            self.total + Self::duration_mul_f32(Instant::now() - self.factor_start, self.factor_value)
        }
    }

    /**
     * Returns elapsed() in seconds as f32.
     */
    pub fn elapsed_f32(self: &Self) -> f32 {
        Self::duration_as_secs(self.elapsed()) as f32
    }

    /**
     * Returns elapsed() in seconds as f64.
     */
    pub fn elapsed_f64(self: &Self) -> f64 {
        Self::duration_as_secs(self.elapsed())
    }

    /**
     * Changes the rate at which time progresses.
     */
    pub fn rate(self: &mut Self, factor: f32) {
        if factor != self.factor_value {
            self.total = self.elapsed();
            self.factor_start = Instant::now();
            self.factor_value = factor;
        }
    }

    /**
     * Sets the elapsed amount of time in this timeframe.
     */
    pub fn set_elapsed(self: &mut Self, elapsed: Duration) {
        self.total = elapsed;
    }

    /**
     * Returns the given Duration in seconds as f64.
     */
    pub fn duration_as_secs(duration: Duration) -> f64 {
        duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
    }

    /**
     * Multiplies the given Duration with given f32 and returns a new Duration.
     */
    fn duration_mul_f32(duration: Duration, multiplier: f32) -> Duration {
        let adjusted_duration = Self::duration_as_secs(duration) * (multiplier as f64);
        Duration::new(adjusted_duration.trunc() as u64, (adjusted_duration.fract() * 1e9) as u32)
    }
}