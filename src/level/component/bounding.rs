use specs;

/**
 * Bounding Box component
 * 
 * Entities with a bounding box collide with each other unless they share a faction.
 */
#[derive(Clone, Debug)]
pub struct Bounding {
    pub radius: f32, // !todo starting out simple
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
