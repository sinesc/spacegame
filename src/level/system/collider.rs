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
    world_state: specs::ReadExpect<'a, WorldState>,
    spatial: specs::WriteStorage<'a, component::Spatial>,
    visual: specs::WriteStorage<'a, component::Visual>,
    lifetime: specs::WriteStorage<'a, component::Lifetime>,
    fading: specs::WriteStorage<'a, component::Fading>,
    bounding: specs::WriteStorage<'a, component::Bounding>,
    hitpoints: specs::WriteStorage<'a, component::Hitpoints>,
    entities: specs::Entities<'a>,
    lazy: specs::Read<'a, specs::LazyUpdate>,
}

impl<'a> Collider {
    pub fn new() -> Self {
        Collider {
        }
    }
}

impl<'a> specs::System<'a> for Collider {
    type SystemData = ColliderData<'a>;

    fn run(&mut self, mut data: ColliderData) {
		use specs::Join;

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
                data.world_state.spawn_lazy(&data.lazy, &data.entities, "explosion", Some(position), None, None);
                rodio::play_raw(&data.world_state.inf.audio, data.world_state.inf.boom.samples());
            }

            if b <= a {
                let position = data.spatial.get(entity_b).unwrap().position;    // !todo can directly pass as parameter due to lameness
                let effect_size = data.visual.get(entity_b).unwrap().effect_size;
                data.world_state.spawn_lazy(&data.lazy, &data.entities, "explosion", Some(position), None, None);
                rodio::play_raw(&data.world_state.inf.audio, data.world_state.inf.boom.samples());
            }

            data.hitpoints.get_mut(entity_a).unwrap().0 -= min(a, b);
            data.hitpoints.get_mut(entity_b).unwrap().0 -= min(a, b);
        }
	}
}
