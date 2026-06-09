
/**
 * Hitpoints component
 *
 * Entities with this component can die from damage.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Hitpoints(pub f32);
