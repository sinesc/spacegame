use prelude::*;
use specs::DenseVecStorage;
use def;

/**
 * Shooter component
 */
#[derive(Clone, Debug, Deserialize, Component)]
pub struct Shooter {
    #[serde(deserialize_with = "::def::periodic_deserialize")]
    #[serde(default = "::def::periodic_default")]
    pub interval: Periodic,
    pub spawner: def::SpawnerId, // TODO: look into serde rename, deserialize this from "spawner" into "spawner_id"
}
