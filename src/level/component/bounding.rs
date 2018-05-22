use specs::DenseVecStorage;

/**
 * Bounding Box component
 *
 * Entities with a bounding box collide with each other unless they share a faction.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Bounding {
    pub radius: f32, // !todo starting out simple
    #[serde(deserialize_with = "::def::faction_deserialize")]
    pub faction: u32,
}

