use prelude::*;
use specs;

/**
 * Spatial component
 *
 * Entities with this component have a position and orientation in space.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Spatial {
    /// Current position
    pub position: Vec2,
    /// Current angle
    pub angle: Angle,
    /// Current lean left/right value
    #[serde(default)]
    pub lean: f32,
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
