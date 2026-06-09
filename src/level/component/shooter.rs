use crate::prelude::*;
use crate::def::SpawnerId;

/**
 * Shooter component
 */
#[derive(Clone, Debug, Deserialize)]
pub struct Shooter {
    #[serde(deserialize_with = "crate::def::periodic_deserialize")]
    #[serde(default = "crate::def::periodic_default")]
    pub interval: Periodic,
    pub spawner: SpawnerId, // TODO: look into serde rename, deserialize this from "spawner" into "spawner_id"
}
