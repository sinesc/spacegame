use specs;
use radiant_rs::Vec2;

#[derive(Clone, Debug)]
pub struct Inertial {
    /// Maximum velocity, needs to be positive.
    pub v_max: Vec2,
    /// Fraction of max velocity currently being applied.
    pub v_fraction: Vec2,
    /// Computed current velocity
    pub v_current: Vec2,
    /// Transition speed when trying to move
    pub trans_motion: f32,
    /// Transition speed when not trying to move
    pub trans_rest: f32
}

impl Inertial {
    pub fn new(mut v_max: Vec2, mut v_fraction: Vec2, trans_motion: f32, trans_rest: f32) -> Self {

        // Move sign from v_max to v_fraction/v_current.
        // v_max should always be positive with v_fraction pointing into the current
        // direction, but it tends to be more convenient to set only v_max on spawn

        if v_max.0 <= 0.0 {
            v_max.0 *= -1.0;
            v_fraction.0 *= -1.0;
        }

        if v_max.1 <= 0.0 {
            v_max.1 *= -1.0;
            v_fraction.1 *= -1.0;
        }

        Inertial {
            v_max       : v_max,
            v_fraction  : v_fraction,
            v_current   : v_max * v_fraction,
            trans_motion: trans_motion,
            trans_rest  : trans_rest,
        }
    }
}

impl specs::Component for Inertial {
    type Storage = specs::VecStorage<Inertial>;
}
