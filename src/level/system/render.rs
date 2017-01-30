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

		let (spatials, mut visuals, faders) = arg.fetch(|w|
			(w.read::<component::Spatial>(), w.write::<component::Visual>(), w.read::<component::Fading>())
		);

        // apply fade effects

		for (fading, mut visual) in (&faders, &mut visuals).iter() {
            if state.age >= fading.start {
                let duration = fading.end - fading.start;
                let progress = state.age - fading.start;
                let alpha = 1.0 - (progress / duration);
                if alpha >= 0.0 {
                    visual.color.set_a(alpha);
                }
            }
        }

        // draw sprites

        let mut num_sprites = 0;

		for (spatial, mut visual) in (&spatials, &mut visuals).iter() {

            state.inf.scene.sprite_transformed(visual.layer_id, visual.sprite_id, visual.frame_id as u32, spatial.position, visual.color, spatial.angle.to_radians(), Point2(1.0, 1.0));

            visual.frame_id = if visual.fps == 0 {
                cmp::min(29, cmp::max(0, (15.0 + (15.0 * spatial.lean)) as i32)) as f32
            } else {
                visual.frame_id + state.delta * visual.fps as f32
            };

            num_sprites += 1;
        }

        state.inf.scene.write(state.inf.layer, state.inf.font, &format!("FPS: {:?}\r\ndelta: {:?}\r\nentities: {:?}", (1.0 / state.delta).floor(), state.delta, num_sprites), Point2(10.0, 10.0));
	}
}
