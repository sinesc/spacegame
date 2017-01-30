use specs;
use radiant_rs::utils::Periodic;

#[derive(Clone, Debug)]
pub struct Shooter {
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
