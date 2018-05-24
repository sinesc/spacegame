#![allow(dead_code)]

use prelude::*;

#[derive(Clone)]
pub struct Timeframe {
    total               : Duration,
    factor_start        : Instant,
    factor_value        : f64,
    lerp_factor_end     : Option<Instant>,
    lerp_factor_value   : f64,
}

impl Timeframe {
    /**
     * Creates a new timeframe.
     */
    pub fn new() -> Timeframe {
        Timeframe {
            total               : Duration::new(0, 0),
            factor_start        : Instant::now(),
            factor_value        : 1.0,
            lerp_factor_end     : None,
            lerp_factor_value   : 1.0,
        }
    }

    /**
     * Returns the current rate at which time progresses.
     */
    pub fn rate(self: &Self) -> f64 {
        if self.lerp_factor_end.is_some() {
            let now = Instant::now();
            self.factor_value + (self.lerp_factor_value - self.factor_value) * self.lerp_progress(now)
        } else {
            self.factor_value
        }
    }

    /**
     * Returns the elapsed duration of time since the timeframe began.
     */
    pub fn elapsed(self: &Self) -> Duration {
        if let Some(lerp_factor_end) = self.lerp_factor_end {
            let now = Instant::now();

            // linear segment
            let factor = self.rate();
            let real_expired = Self::duration_to_secs(min(now, lerp_factor_end) - self.factor_start);
            let mut total = 0.5 * self.factor_value * real_expired + 0.5 * factor * real_expired;

            // normal progression past the linear segment
            if now > lerp_factor_end {
                total += Self::duration_to_secs(now - lerp_factor_end) * self.lerp_factor_value;
            }

            self.total + Self::duration_from_secs(total)

        } else if self.factor_value == 1.0 {
            self.total + (Instant::now() - self.factor_start)
        } else if self.factor_value == 0.0 {
            self.total
        } else {
            self.total + Self::duration_mul_f64(Instant::now() - self.factor_start, self.factor_value)
        }
    }

    /**
     * Returns elapsed() in seconds as f32.
     */
    pub fn elapsed_f32(self: &Self) -> f32 {
        Self::duration_to_secs(self.elapsed()) as f32
    }

    /**
     * Returns elapsed() in seconds as f64.
     */
    pub fn elapsed_f64(self: &Self) -> f64 {
        Self::duration_to_secs(self.elapsed())
    }

    /**
     * Changes the rate at which time progresses.
     */
    pub fn set_rate(self: &mut Self, rate: f64) {
        if self.lerp_factor_end.is_some() || rate != self.factor_value {
            self.total = self.elapsed();
            self.factor_start = Instant::now();
            self.factor_value = rate;
            self.lerp_factor_end = None;
        }
    }

    /**
     * Continuously changes the rate at which time progresses (accelerate time).
     */
    pub fn lerp_rate(self: &mut Self, target_rate: f64, mut real_duration: Duration) {

        let is_lerp = self.lerp_factor_end.is_some();

        if (is_lerp && target_rate != self.lerp_factor_value) || (!is_lerp && target_rate != self.factor_value) {

            let now = Instant::now();
            let current_factor = self.rate();

            // if still lerping, compute expected rate of change (that would have occured without unfinished lerp) and
            // shorten duration of this new lerp to match that rate
            if is_lerp && current_factor != self.lerp_factor_value {
                let distance_to_expected_target = (self.lerp_factor_value - target_rate).abs();
                let distance_to_actual_target = (current_factor - target_rate).abs();
                real_duration = Self::duration_mul_f64(real_duration, distance_to_actual_target / distance_to_expected_target);
            }

            self.total = self.elapsed();
            self.factor_value = current_factor;
            self.factor_start = now;
            self.lerp_factor_end = Some(self.factor_start + real_duration);
            self.lerp_factor_value = target_rate;
        }
    }

    /**
     * Returns whether the timeframe is currently lerping.
     */
    pub fn is_lerp(self: &Self) -> bool {
        if let Some(lerp_factor_end) = self.lerp_factor_end {
            lerp_factor_end < Instant::now()
        } else {
            false
        }
    }

    /**
     * Sets the elapsed amount of time in this timeframe.
     * Instantly finishes active lerp_rate(), if any.
     */
    pub fn set_elapsed(self: &mut Self, elapsed: Duration) {
        self.total = elapsed;
        self.factor_start = Instant::now();
        if self.lerp_factor_end.is_some() {
            self.factor_value = self.lerp_factor_value;
            self.lerp_factor_end = None;
        }
    }

    /**
     * Returns the given Duration in seconds as f64.
     */
    pub fn duration_to_secs(duration: Duration) -> f64 {
        duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
    }

    pub fn duration_from_secs(secs: f64) -> Duration {
        Duration::new(secs.trunc() as u64, (secs.fract() * 1e9) as u32)
    }

    /**
     * Multiplies the given Duration with given f64 and returns a new Duration.
     */
    fn duration_mul_f64(duration: Duration, multiplier: f64) -> Duration {
        Self::duration_from_secs(Self::duration_to_secs(duration) * multiplier)
    }

    /**
     * Returns progress from 0 to 1 of the current lerp
     */
    fn lerp_progress(self: &Self, now: Instant) -> f64 {
        if let Some(lerp_factor_end) = self.lerp_factor_end {
            if now < lerp_factor_end {
                let real_expired = Self::duration_to_secs(min(now, lerp_factor_end) - self.factor_start);
                let real_total = Self::duration_to_secs(lerp_factor_end - self.factor_start);
                real_expired / real_total
            } else {
                1.
            }
        } else {
            panic!("lerp_progress called outside of lerp");
        }
    }
}