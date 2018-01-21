use specs;
use radiant_rs::math::*;

/**
 * Spatial component
 * 
 * Entities with this component have a position and orientation in space.
 */
#[derive(Clone, Debug)]
pub struct Spatial {
    /// Current position
    pub position: Vec2,
    /// Current angle
    pub angle: Angle,
    /// Current lean left/right value
    pub lean: f32, // todo: this isn't ideal here either
}

impl Spatial {
    pub fn new(position: Vec2, angle: Angle) -> Self {
        Spatial {
            position    : position,
            angle       : angle,
            lean        : 0.0,
        }
    }
}

impl specs::Component for Spatial {
    type Storage = specs::VecStorage<Spatial>;
}
