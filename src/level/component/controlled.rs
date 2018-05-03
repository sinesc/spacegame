use specs;

/**
 * Controlled component (Player)
 * 
 * Entities with this component are controlled by a player.
 */
#[derive(Clone, Debug)]
pub struct Controlled {
    /// Input mapping id.
    pub input_id: u32,
}

impl Controlled {
    pub fn new(input_id: u32) -> Self {
        Controlled {
            input_id: input_id,
        }
    }
}

impl specs::Component for Controlled {
    type Storage = specs::HashMapStorage<Controlled>;
}
