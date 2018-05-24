use specs::DenseVecStorage;
use def::FactionId;
use def::SpawnerId;

/**
 * Powerup component
 *
 * Powerups collide with the given faction.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Powerup {
    pub radius: f32, // !todo starting out simple
    pub faction: FactionId,
    pub spawner: SpawnerId,
}

