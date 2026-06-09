
/**
 * Lifetime component
 *
 * Entities with this component expire after given amount of time.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Lifetime(pub f32);
