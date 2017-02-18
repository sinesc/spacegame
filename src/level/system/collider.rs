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

		let (mut spatials, mut visuals, mut lifetimes, mut faders, boundings, entities) = arg.fetch(|w| (
            w.write::<component::Spatial>(),
            w.write::<component::Visual>(),
            w.write::<component::Lifetime>(),
            w.write::<component::Fading>(),
            w.read::<component::Bounding>(),
            w.entities()
        ));

        let mut explosions = Vec::new();

		for (spatial_a, bounding_a, entity_a) in (&spatials, &boundings, &entities).iter() {

            for (spatial_b, bounding_b, entity_b) in (&spatials, &boundings, &entities).iter() {

                if bounding_a.faction != bounding_b.faction
                    && entity_a != entity_b
                    && bounding_a.radius + bounding_a.radius > spatial_a.position.distance(&spatial_b.position) {
                    arg.delete(entity_a);
                    arg.delete(entity_b);
                    explosions.push(spatial_a.position);
                }
            }
		}

        let mut spawn = |origin: Point2| {
            let explosion = arg.create();
            spatials.insert(explosion, component::Spatial::new(origin, Angle(0.0), false));
            visuals.insert(explosion, component::Visual::new(state.inf.effects.clone(), Some(state.inf.bloom.clone()), state.inf.explosion.clone(), Color::white(), 30));
            lifetimes.insert(explosion, component::Lifetime(state.age + 1.0));
            faders.insert(explosion, component::Fading::new(state.age + 0.5, state.age + 1.0));
        };

        for position in explosions {
            spawn(position);
        }
	}
}
