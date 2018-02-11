use prelude::*;
use specs;

/**
 * Inertial component
 * 
 * Entities with this component accellerate/rotate towards given vector according to trans_motion/rest values.
 */
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
    /// Transition speed when trying to stop
    pub trans_rest: f32,
    /// True if the angle should not be updated from inertial.v_current
    pub angle_locked: bool, // todo: use a factor here, so it can be set to 0 to lock or anything else to control how fast rotation happens
}

impl Inertial {
    pub fn new(mut v_max: Vec2, mut v_fraction: Vec2, trans_motion: f32, trans_rest: f32, angle_locked: bool) -> Self {

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
            angle_locked: angle_locked,
        }
    }
}

impl specs::Component for Inertial {
    type Storage = specs::VecStorage<Inertial>;
}
