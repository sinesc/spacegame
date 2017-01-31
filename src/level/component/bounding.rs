use specs;
use radiant_rs::*;

#[derive(Clone, Debug)]
pub struct Bounding {
    pub radius: f32, // !todo staring out simple
    pub faction: u32,
}

impl Bounding {
    pub fn new(radius: f32, faction: u32) -> Self {
        Bounding {
            radius: radius,
            faction: faction,
        }
    }
}

impl specs::Component for Bounding {
    type Storage = specs::VecStorage<Bounding>;
}
