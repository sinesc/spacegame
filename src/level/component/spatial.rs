use specs;
use radiant_rs::*;
use radiant_rs::math::*;

#[derive(Clone, Debug)]
pub struct Spatial {
    /// Current position
    pub position: Vec2,
    /// Current angle
    pub angle: Angle,
    /// Current lean left/right value
    pub lean: f32,
    /// True if the angle should not be updated from inertial.v_current
    pub angle_locked: bool,
}

impl Spatial {
    pub fn new(position: Vec2, angle: Angle, angle_locked: bool) -> Self {
        Spatial {
            position    : position,
            angle       : angle,
            lean        : 0.0,
            angle_locked: angle_locked,
        }
    }
}

impl specs::Component for Spatial {
    type Storage = specs::VecStorage<Spatial>;
}
