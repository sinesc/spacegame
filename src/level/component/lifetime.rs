use specs;

/**
 * Lifetime component
 *
 * Entities with this component expire after given amount of time.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Lifetime(pub f32);

impl specs::Component for Lifetime {
    type Storage = specs::VecStorage<Lifetime>;
}
