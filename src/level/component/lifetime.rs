
/**
 * Lifetime component
 *
 * Entities with this component expire after given amount of time.
 */
#[derive(Clone, Debug, Default)]
pub struct Lifetime(pub f32);
