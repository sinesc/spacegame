use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;
use radiant_rs::math::*;

pub struct Collider {
}

impl<'a> Collider {
    pub fn new() -> Self {
        Collider {
        }
    }
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

impl<'a> specs::System<'a> for Collider {
    type SystemData = ColliderData<'a>;

    fn run(&mut self, mut data: ColliderData) {
		use specs::Join;

        // dirty test all against all other entities

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

        let mut explosions = Vec::new();

        for (entity_a, entity_b) in collisions {

            let a = data.hitpoints.get(entity_a).unwrap().0;
            let b = data.hitpoints.get(entity_b).unwrap().0;
            let value = utils::min(a, b);

            if a <= 0. {
                explosions.push((data.spatial.get(entity_a).unwrap().position, data.visual.get(entity_a).unwrap().effect_size));
                data.entities.delete(entity_a).unwrap();
            } else {
                data.hitpoints.get_mut(entity_a).unwrap().0 -= value;
            }

            if b <= 0. {
                explosions.push((data.spatial.get(entity_b).unwrap().position, data.visual.get(entity_b).unwrap().effect_size));
                data.entities.delete(entity_b).unwrap();
            } else {
                data.hitpoints.get_mut(entity_b).unwrap().0 -= value;
            }
        }

        let mut spawn = |origin: Vec2, effect_size: f32| {
            let explosion = data.entities.create();
            data.spatial.insert(explosion, component::Spatial::new(origin, Angle(0.0), false));
            data.visual.insert(explosion, component::Visual::new(None, Some(data.world_state.inf.effects.clone()), data.world_state.inf.explosion.clone(), Color::WHITE, 30, effect_size));
            data.lifetime.insert(explosion, component::Lifetime(data.world_state.age + 1.0));
            data.fading.insert(explosion, component::Fading::new(data.world_state.age + 0.5, data.world_state.age + 1.0));
        };

        for (position, effect_size) in explosions {
            spawn(position, effect_size);
        }
	}
}
