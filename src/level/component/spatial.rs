use crate::prelude::*;

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
