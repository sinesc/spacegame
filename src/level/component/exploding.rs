use specs::DenseVecStorage;
use def::SpawnerId;

/**
 * Exploding component
 *
 * Entities with this component explode on destruction.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Exploding {
    //pub start_time: f32,     // todo: not yes used. overlay multiple explosions for given duration
    //pub duration: f32,
    #[serde(deserialize_with = "::def::spawner_deserialize")]
    pub spawner: SpawnerId,
}
