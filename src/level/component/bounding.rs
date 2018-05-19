use specs;

/**
 * Bounding Box component
 *
 * Entities with a bounding box collide with each other unless they share a faction.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Bounding {
    pub radius: f32, // !todo starting out simple
    #[serde(deserialize_with = "::def::entity::faction_deserialize")]
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
