use specs::DenseVecStorage;

/**
 * Hitpoints component
 *
 * Entities with this component can die from damage.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Hitpoints(pub f32);
