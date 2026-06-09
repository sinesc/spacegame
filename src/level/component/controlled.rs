
/**
 * Controlled component (Player)
 *
 * Entities with this component are controlled by a player.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Controlled {
    /// Input mapping id.
    pub input_id: u32,
}

