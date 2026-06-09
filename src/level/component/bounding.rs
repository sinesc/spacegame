use crate::def::FactionId;

/**
 * Bounding Box component
 *
 * Entities with a bounding box collide with each other unless they share a faction.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Bounding {
    pub radius: f32, // !todo starting out simple
    pub faction: FactionId,
}

