use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;
use std::cmp;

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

        let mut num_sprites = 0;

		for (spatial, mut visual) in (&spatials, &mut visuals).iter() {

            state.inf.scene.sprite_transformed(visual.layer_id, visual.sprite_id, visual.frame_id as u32, spatial.position.0, spatial.position.1, Color::white(), spatial.angle, 1.0, 1.0);

            visual.frame_id = if visual.fps == 0 {
                cmp::min(29, cmp::max(0, (15.0 + (15.0 * spatial.lean)) as i32)) as f32
            } else {
                visual.frame_id + 1.0
            };

            num_sprites += 1;
		}

        state.inf.scene.write(state.inf.layer, state.inf.font, &format!("entities: {:?}", num_sprites), 10.0, 1.0);
	}
}
