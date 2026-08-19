use crate::prelude::*;

/**
 * Visual component
 *
 * Entities with this component are rendered.
 */
#[derive(Clone, Debug)]
pub struct Visual {
    pub layer           : Option<Arc<Layer>>,
    pub effect_layer    : Option<Arc<Layer>>,
    pub sprite          : Arc<Sprite>,
    pub scale           : f32,
    pub effect_scale    : f32,
    pub color           : Color,
    pub effect_color    : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}

/**
 * Spatial component
 *
 * Entities with this component have a position and orientation in space.
 */
#[derive(Clone, Debug, Default)]
pub struct Spatial {
    /// Current position
    pub position: Vec2,
    /// Current angle
    pub angle: Angle,
    /// Current lean left/right value
    pub lean: f32,
}

/**
 * Inertial component motion type
 */
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, Default)]
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

/**
 * Lifetime component
 *
 * Entities with this component expire after given amount of time.
 */
#[derive(Clone, Debug, Default)]
pub struct Lifetime(pub f32);

/**
 * Fading component
 *
 * Entities with this component fade after a certain amount of time.
 */
#[derive(Clone, Debug, Default)]
pub struct Fading {
    //pub value: f32,
    pub start: f32,
    pub end: f32,
}

/**
 * Bounding Box component
 *
 * Entities with a bounding box collide with each other unless they share a faction.
 */
#[derive(Clone, Debug, Default)]
pub struct Bounding {
    pub radius: f32, // !todo starting out simple
    pub faction: u16,
}

/**
 * Hitpoints component
 *
 * Entities with this component can die from damage.
 */
#[derive(Clone, Debug, Default)]
pub struct Hitpoints(pub f32);

/**
 * Script component
 *
 * Entities with this component participate in scripted entity logic.
 * The `u16` value is the script type/behavior ID for dispatch in Itsy code.
 */
#[derive(Clone, Debug, Default)]
pub struct Script(pub u16);
