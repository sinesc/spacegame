use prelude::*;
use specs;
use level::component;
use level::WorldState;

/**
 * Collider system
 *
 * This system detects colliding entities with a Bounding component and applies damage.
 */
pub struct Collider;

#[derive(SystemData)]
pub struct ColliderData<'a> {
    world_state: specs::ReadExpect<'a, WorldState>,
    spatial: specs::ReadStorage<'a, component::Spatial>,
    bounding: specs::ReadStorage<'a, component::Bounding>,
    hitpoints: specs::WriteStorage<'a, component::Hitpoints>,
    exploding: specs::ReadStorage<'a, component::Exploding>,
    entities: specs::Entities<'a>,
    lazy: specs::Read<'a, specs::LazyUpdate>,
}

impl<'a> specs::System<'a> for Collider {
    type SystemData = ColliderData<'a>;

    fn run(&mut self, mut data: ColliderData) {
		use specs::Join;

        // test all against all other entities todo: use a grid or quadtree to reduce checks

        let mut collisions = Vec::new();

		for (spatial_a, bounding_a, _, entity_a) in (&data.spatial, &data.bounding, &data.hitpoints, &*data.entities).join() {
            for (spatial_b, bounding_b, _, entity_b) in (&data.spatial, &data.bounding, &data.hitpoints, &*data.entities).join() {

                if bounding_a.faction != bounding_b.faction
                    && entity_a != entity_b
                    && bounding_a.radius + bounding_a.radius > spatial_a.position.distance(&spatial_b.position) {

                    collisions.push((entity_a, entity_b, spatial_a.position, spatial_b.position));
                }
            }
		}

        for (entity_a, entity_b, position_a, position_b) in collisions {

            let a = data.hitpoints.get(entity_a).unwrap().0;
            let b = data.hitpoints.get(entity_b).unwrap().0;

            if a <= b {
                if let Some(exploding) = data.exploding.get(entity_a) {
                    data.world_state.spawner(&data.lazy, &data.entities, exploding.spawner, Angle(0.), Some(position_a), None, None);
                }
            } else if b <= a {
                if let Some(exploding) = data.exploding.get(entity_b) {
                    data.world_state.spawner(&data.lazy, &data.entities, exploding.spawner, Angle(0.), Some(position_b), None, None);
                }
            }

            data.hitpoints.get_mut(entity_a).unwrap().0 -= min(a, b);
            data.hitpoints.get_mut(entity_b).unwrap().0 -= min(a, b);
        }
	}
}
