use specs;

/**
 * Controlled component
 * 
 * Entities with this component are controlled by a player.
 */
#[derive(Clone, Debug)]
pub struct Controlled {
    /// Input mapping id.
    pub input_id: u32,
    /// Current angular velocity.
    pub av_current: f32,
    /// Rate of change for angular velocity.
    pub av_trans: f32,
    /// Maximum angular velocity.
    pub av_max: f32,
}

impl Controlled {
    pub fn new(input_id: u32) -> Self {
        Controlled {
            input_id: input_id,
            av_current: 0.0,
            av_trans: 5.0,
            av_max: 10.0,
        }
    }
}

impl specs::Component for Controlled {
    type Storage = specs::HashMapStorage<Controlled>;
}
