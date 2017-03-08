use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;

pub struct Collider {
}

impl<'a> Collider {
    pub fn new() -> Self {
        Collider {
        }
    }
}

impl<'a> specs::System<WorldState> for Collider {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (mut spatials, mut visuals, mut lifetimes, mut hitpoints, mut faders, boundings, entities) = arg.fetch(|w| (
            w.write::<component::Spatial>(),
            w.write::<component::Visual>(),
            w.write::<component::Lifetime>(),
            w.write::<component::Hitpoints>(),
            w.write::<component::Fading>(),
            w.read::<component::Bounding>(),
            w.entities()
        ));

        // dirty test all against all other entities

        let mut collisions = Vec::new();

		for (spatial_a, bounding_a, _, _, entity_a) in (&spatials, &boundings, &visuals, &hitpoints, &entities).iter() {
            for (spatial_b, bounding_b, _, _, entity_b) in (&spatials, &boundings, &visuals, &hitpoints, &entities).iter() {

                if bounding_a.faction != bounding_b.faction
                    && entity_a != entity_b
                    && bounding_a.radius + bounding_a.radius > spatial_a.position.distance(&spatial_b.position) {

                    collisions.push((entity_a, entity_b));
                }
            }
		}

        let mut explosions = Vec::new();

        for (entity_a, entity_b) in collisions {

            let a = hitpoints.get(entity_a).unwrap().0;
            let b = hitpoints.get(entity_b).unwrap().0;
            let value = utils::min(a, b);

            if a <= 0. {
                explosions.push((spatials.get(entity_a).unwrap().position, visuals.get(entity_a).unwrap().effect_size));
                arg.delete(entity_a);
            } else {
                hitpoints.get_mut(entity_a).unwrap().0 -= value;
            }

            if b <= 0. {
                explosions.push((spatials.get(entity_b).unwrap().position, visuals.get(entity_b).unwrap().effect_size));
                arg.delete(entity_b);
            } else {
                hitpoints.get_mut(entity_b).unwrap().0 -= value;
            }
        }

        let mut spawn = |origin: Vec2, effect_size: f32| {
            let explosion = arg.create();
            spatials.insert(explosion, component::Spatial::new(origin, Angle(0.0), false));
            visuals.insert(explosion, component::Visual::new(None, Some(state.inf.effects.clone()), state.inf.explosion.clone(), Color::white(), 30, effect_size));
            lifetimes.insert(explosion, component::Lifetime(state.age + 1.0));
            faders.insert(explosion, component::Fading::new(state.age + 0.5, state.age + 1.0));
        };

        for (position, effect_size) in explosions {
            spawn(position, effect_size);
        }
	}
}
