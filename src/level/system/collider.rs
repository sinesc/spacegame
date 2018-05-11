use prelude::*;
use specs;
use rodio;
use level::component;
use level::WorldState;

/**
 * Collider system
 * 
 * This system detects colliding entities with a Bounding component and applies damage.
 * todo: move effect of this collision somewhere else. find out how.
 */
pub struct Collider {
}

#[derive(SystemData)]
pub struct ColliderData<'a> {
    world_state: specs::Fetch<'a, WorldState>,
    spatial: specs::WriteStorage<'a, component::Spatial>,
    visual: specs::WriteStorage<'a, component::Visual>,
    lifetime: specs::WriteStorage<'a, component::Lifetime>,
    fading: specs::WriteStorage<'a, component::Fading>,
    bounding: specs::WriteStorage<'a, component::Bounding>,
    hitpoints: specs::WriteStorage<'a, component::Hitpoints>,
    entities: specs::Entities<'a>,
}

impl<'a> Collider {
    pub fn new() -> Self {
        Collider {
        }
    }

    fn spawn<'b>(data: &mut ColliderData<'b>, origin: Vec2, effect_size: f32) {
        let explosion = data.entities.create();
        let age = data.world_state.age.elapsed_f32();
        data.spatial.insert(explosion, component::Spatial::new(origin, Angle(0.0)));
        data.visual.insert(explosion, component::Visual::new(None, Some(data.world_state.inf.layer["effects"].clone()), data.world_state.inf.explosion.clone(), Color::WHITE, 1.0, 30, effect_size));
        data.lifetime.insert(explosion, component::Lifetime(age + 1.0));
        data.fading.insert(explosion, component::Fading::new(age + 0.5, age + 1.0));
    }
}

impl<'a> specs::System<'a> for Collider {
    type SystemData = ColliderData<'a>;

    fn run(&mut self, mut data: ColliderData) {
		use specs::Join;

        if data.world_state.paused {
            return;
        }
        
        // test all against all other entities todo: use a grid or quadtree to reduce checks

        let mut collisions = Vec::new();

		for (spatial_a, bounding_a, _, _, entity_a) in (&data.spatial, &data.bounding, &data.visual, &data.hitpoints, &*data.entities).join() {
            for (spatial_b, bounding_b, _, _, entity_b) in (&data.spatial, &data.bounding, &data.visual, &data.hitpoints, &*data.entities).join() {

                if bounding_a.faction != bounding_b.faction
                    && entity_a != entity_b
                    && bounding_a.radius + bounding_a.radius > spatial_a.position.distance(&spatial_b.position) {

                    collisions.push((entity_a, entity_b));
                }
            }
		}

        for (entity_a, entity_b) in collisions {

            let a = data.hitpoints.get(entity_a).unwrap().0;
            let b = data.hitpoints.get(entity_b).unwrap().0;

            if a <= b {
                let position = data.spatial.get(entity_a).unwrap().position;    // !todo can directly pass as parameter due to lameness
                let effect_size = data.visual.get(entity_a).unwrap().effect_size;
                Self::spawn(&mut data, position, effect_size);
                rodio::play_raw(&data.world_state.inf.audio, data.world_state.inf.boom.samples());
            }

            if b <= a {
                let position = data.spatial.get(entity_b).unwrap().position;    // !todo can directly pass as parameter due to lameness
                let effect_size = data.visual.get(entity_b).unwrap().effect_size;
                Self::spawn(&mut data, position, effect_size);
                rodio::play_raw(&data.world_state.inf.audio, data.world_state.inf.boom.samples());
            }

            data.hitpoints.get_mut(entity_a).unwrap().0 -= min(a, b);
            data.hitpoints.get_mut(entity_b).unwrap().0 -= min(a, b);
        }
	}
}
