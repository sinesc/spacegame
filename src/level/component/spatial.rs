use prelude::*;
use specs::DenseVecStorage;

/**
 * Spatial component
 *
 * Entities with this component have a position and orientation in space.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Spatial {
    /// Current position
    pub position: Vec2,
    /// Current angle
    pub angle: Angle,
    /// Current lean left/right value
    #[serde(default)]
    pub lean: f32,
}
