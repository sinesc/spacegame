use specs;

/**
 * Hitpoints component
 *
 * Entities with this component can die from damage.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Hitpoints(pub f32);

impl Hitpoints {
    pub fn new(value: f32) -> Self {
        Hitpoints(value)
    }
}

impl specs::Component for Hitpoints {
    type Storage = specs::VecStorage<Hitpoints>;
}
