use specs;
use level::component;
use level::WorldState;

/**
 * Cleanup system
 *
 * This system removes dead/expired entities
 */
pub struct Cleanup {
}

impl<'a> Cleanup {
    pub fn new() -> Self {
        Cleanup {
        }
    }
}

#[derive(SystemData)]
pub struct CleanupData<'a> {
    world_state: specs::Fetch<'a, WorldState>,
    lifetime: specs::ReadStorage<'a, component::Lifetime>,
    hitpoints: specs::ReadStorage<'a, component::Hitpoints>,
    entities: specs::Entities<'a>,
}


impl<'a> specs::System<'a> for Cleanup {
    type SystemData = CleanupData<'a>;

    fn run(&mut self, data: CleanupData) {
		use specs::Join;

		for (lifetime, entity) in (&data.lifetime, &*data.entities).join() {
            if lifetime.0 < data.world_state.age {
                data.entities.delete(entity).unwrap();
            }
		}

		for (hitpoints, entity) in (&data.hitpoints, &*data.entities).join() {
            if hitpoints.0 <= 0. {
                data.entities.delete(entity).unwrap();
            }
		}
	}
}
