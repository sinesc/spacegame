use prelude::*;
use specs::DenseVecStorage;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum InertialMotionType {
    Const,
    FollowVector,
    StrafeVector,
    Detached
}

impl Default for InertialMotionType {
    fn default() -> InertialMotionType {
        InertialMotionType::FollowVector
    }
}

/**
 * Inertial component
 *
 * Entities with this component accellerate/rotate towards given vector according to trans_motion/rest values.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
#[serde(default)]
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
