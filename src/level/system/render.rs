use crate::prelude::*;
use hecs;
use crate::level::component;
use crate::level::WorldState;
use std::cmp;

pub struct Render {
    fps_interval: Periodic,
    num_frames: u32,
    last_num_frames: u32,
}

impl Render {
    pub fn new() -> Self {
        Render {
            fps_interval: Periodic::new(0.0, 1.0),
            num_frames: 0,
            last_num_frames: 0,
        }
    }

    pub fn run(&mut self, world: &mut hecs::World, ws: &WorldState) {
        let age = ws.age;
        let mut num_sprites = 0;

        for (_entity, (spatial, visual, fading)) in world.query_mut::<(
            &component::Spatial,
            &mut component::Visual,
            Option<&component::Fading>,
        )>() {
            if let Some(fading) = fading {
                if age >= fading.start {
                    let duration = fading.end - fading.start;
                    let progress = age - fading.start;
                    let alpha = 1.0 - (progress / duration);
                    if alpha >= 0.0 {
                        visual.color.set_a(alpha);
                        visual.effect_color.set_a(alpha);
                    }
                }
            }

            if let Some(ref layer) = visual.layer {
                visual.sprite.draw_transformed(
                    &layer, visual.frame_id as u32,
                    spatial.position, visual.color.to_pm(),
                    spatial.angle.to_radians(), (visual.scale, visual.scale)
                );
            }

            if let Some(ref effect_layer) = visual.effect_layer {
                visual.sprite.draw_transformed(
                    &effect_layer, visual.frame_id as u32,
                    spatial.position, visual.effect_color.to_pm(),
                    spatial.angle.to_radians(), (visual.effect_scale, visual.effect_scale)
                );
            }

            visual.frame_id = if visual.fps == 0 {
                cmp::min(29, cmp::max(0, (15.0 + (15.0 * spatial.lean)) as i32)) as f32
            } else {
                visual.frame_id + ws.delta * visual.fps as f32
            };

            num_sprites += 1;
        }

        self.num_frames += 1;

        if self.fps_interval.elapsed(age) {
            self.last_num_frames = self.num_frames;
            self.num_frames = 0;
        }

        ws.inf.font.write(&ws.inf.layer["text"], &format!("Entities: {:?}", num_sprites), (10.0, 72.0), Color::alpha_pm(0.4));
    }
}
