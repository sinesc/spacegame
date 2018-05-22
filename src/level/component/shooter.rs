use prelude::*;
use specs::DenseVecStorage;

/**
 * Shooter component
 *
 * todo: This is a stupid component. I need to find a better solution.
 */
#[derive(Clone, Debug, Deserialize, Component)]
pub struct Shooter {
    #[serde(deserialize_with = "::def::periodic_deserialize")]
    #[serde(default = "::def::periodic_default")]
    pub interval: Periodic,
    #[serde(deserialize_with = "::def::spawner_deserialize")]
    pub spawner: usize, // TODO: look into serde rename, deserialize this from "spawner" into "spawner_id"
}
