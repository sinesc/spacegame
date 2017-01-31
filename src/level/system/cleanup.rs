use specs;
use level::component;
use level::WorldState;

pub struct Cleanup {
}

impl<'a> Cleanup {
    pub fn new() -> Self {
        Cleanup {
        }
    }
}

impl<'a> specs::System<WorldState> for Cleanup {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (lifetimes, entities) = arg.fetch(|w|
			(w.read::<component::Lifetime>(), w.entities())
		);

		for (lifetime, entity) in (&lifetimes, &entities).iter() {
            if lifetime.0 < state.age {
                arg.delete(entity);
            }
		}

	}
}
