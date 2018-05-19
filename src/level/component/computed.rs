use specs;

/**
 * Computed component (NPC)
 *
 * Entities with this component are controlled by the game.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Computed {

}

impl Computed {
    pub fn new() -> Self {
        Computed {

        }
    }
}

impl specs::Component for Computed {
    type Storage = specs::VecStorage<Computed>;
}
