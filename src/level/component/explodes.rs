use specs::DenseVecStorage;
use def::SpawnerId;

/**
 * Entities with this component explode on destruction, creating an Exploding entity
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Explodes {
    pub spawner: SpawnerId,
}
