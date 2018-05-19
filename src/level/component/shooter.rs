use prelude::*;
use specs;

/**
 * Shooter component
 *
 * todo: This is a stupid component. I need to find a better solution.
 */
#[derive(Clone, Debug, Deserialize)]
pub struct Shooter {
    #[serde(deserialize_with = "::def::shared::periodic_deserialize")]
    #[serde(default = "::def::shared::periodic_default")]
    pub interval: Periodic,
}

impl Shooter {
    pub fn new(interval: f32) -> Self {
        Shooter {
            interval: Periodic::new(0.0, interval),
        }
    }
}

impl specs::Component for Shooter {
    type Storage = specs::VecStorage<Shooter>;
}
