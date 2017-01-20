use std::sync::Arc;
use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;
use radiant_rs::scene::*;

pub struct Render {
}

impl<'a> Render {
    pub fn new() -> Self {
        Render {
        }
    }
}

impl<'a> specs::System<WorldState> for Render {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (spatials, mut visuals) = arg.fetch(|w|
			(w.read::<component::Spatial>(), w.write::<component::Visual>())
		);

		for (spatial, mut visual) in (&spatials, &mut visuals).iter() {
            state.inf.scene.sprite(visual.layer_id, visual.sprite_id, visual.frame_id, spatial.pos.0, spatial.pos.1, Color::white());
            visual.frame_id += 1;
		}
	}
}
