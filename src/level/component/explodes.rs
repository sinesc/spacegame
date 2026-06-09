use crate::def::SpawnerId;

/**
 * Entities with this component explode on destruction, creating an Exploding entity
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Explodes {
    pub spawner: SpawnerId,
}
