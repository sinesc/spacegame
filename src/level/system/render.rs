use prelude::*;
use specs;
use level::component;
use level::WorldState;
use std::cmp;

/**
 * Render system
 *
 * Draws entities with a Visual and Spatial component.
 */
pub struct Render {
    fps_interval: Periodic,
    num_frames: u32,
    last_num_frames: u32,
}

impl<'a> Render {
    pub fn new() -> Self {
        Render {
            fps_interval: Periodic::new(0.0, 1.0),
            num_frames: 0,
            last_num_frames: 0,
        }
    }
}

#[derive(SystemData)]
pub struct RenderData<'a> {
    world_state: specs::Fetch<'a, WorldState>,
    spatial: specs::ReadStorage<'a, component::Spatial>,
    visual: specs::WriteStorage<'a, component::Visual>,
    fading: specs::ReadStorage<'a, component::Fading>,
}

impl<'a> specs::System<'a> for Render {
    type SystemData = RenderData<'a>;

    fn run(&mut self, mut data: RenderData) {
		use specs::Join;

        let age = data.world_state.age.elapsed_f32();

        // apply fade effects

		for (fading, visual) in (&data.fading, &mut data.visual).join() {
            if age >= fading.start {
                let duration = fading.end - fading.start;
                let progress = age - fading.start;
                let alpha = 1.0 - (progress / duration);
                if alpha >= 0.0 {
                    visual.color.set_a(alpha);
                }
            }
        }

        // draw sprites

        let mut num_sprites = 0;

		for (spatial, visual) in (&data.spatial, &mut data.visual).join() {

            if let Some(ref layer) = visual.layer {
                visual.sprite.draw_transformed(&layer, visual.frame_id as u32, spatial.position, visual.color.to_pm(), spatial.angle.to_radians(), (visual.scale, visual.scale));
            }

            if let Some(ref effect_layer) = visual.effect_layer {
                visual.sprite.draw_transformed(&effect_layer, visual.frame_id as u32, spatial.position, visual.color.to_pm(), spatial.angle.to_radians(), (visual.effect_size, visual.effect_size));
            }

            visual.frame_id = if visual.fps == 0 {
                cmp::min(29, cmp::max(0, (15.0 + (15.0 * spatial.lean)) as i32)) as f32
            } else {
                visual.frame_id + data.world_state.delta * visual.fps as f32
            };

            num_sprites += 1;
        }

        self.num_frames += 1;

        if self.fps_interval.elapsed(age) {
            self.last_num_frames = self.num_frames;
            self.num_frames = 0;
        }

        data.world_state.inf.font.write(&data.world_state.inf.layer["text"], &format!("Entities: {:?}", num_sprites), (10.0, 72.0), Color::alpha_pm(0.4));
	}
}
