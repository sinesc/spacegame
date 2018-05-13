use prelude::*;
use specs;

#[derive(Clone, Debug, PartialEq)]
pub enum InertialMotionType {
    FollowVector,
    StrafeVector,
    Detached
}

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

    /// Maximum angular velocity at v_current = 0
    pub av_max_v0: f32,
    /// Maximum angular velocity at v_current = v_max
    pub av_max_vmax: f32,

    /// Rate of change for lean
    pub trans_lean: f32,

    /// Motion type
    pub motion_type: InertialMotionType,
}

impl Inertial {
    pub fn new(mut v_max: Vec2, mut v_fraction: Vec2, av_max: f32) -> Self {

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
            trans_motion: 6.0,
            trans_rest  : 3.0,
            av_max_v0   : av_max,
            av_max_vmax : av_max * 0.2,
            trans_lean  : 10.0,
            motion_type : InertialMotionType::FollowVector,
        }
    }
}

impl specs::Component for Inertial {
    type Storage = specs::VecStorage<Inertial>;
}
