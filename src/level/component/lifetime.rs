use specs::DenseVecStorage;

/**
 * Lifetime component
 *
 * Entities with this component expire after given amount of time.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Lifetime(pub f32);
