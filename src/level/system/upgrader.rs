use prelude::*;
use specs;
use level::component;
use level::WorldState;

/**
 * Upgrader system
 *
 * TODO: doc
 */
pub struct Upgrader;

#[derive(SystemData)]
pub struct UpgraderData<'a> {
    spatial: specs::ReadStorage<'a, component::Spatial>,
    bounding: specs::ReadStorage<'a, component::Bounding>,
    powerup: specs::ReadStorage<'a, component::Powerup>,
    shooter: specs::WriteStorage<'a, component::Shooter>,
    entities: specs::Entities<'a>,
}

impl<'a> specs::System<'a> for Upgrader {
    type SystemData = UpgraderData<'a>;

    fn run(&mut self, mut data: UpgraderData) {
		use specs::Join;

        // test all against all other entities todo: use a grid or quadtree to reduce checks

		for (spatial_a, powerup, entity_a) in (&data.spatial, &data.powerup, &*data.entities).join() {
            for (spatial_b, bounding, entity_b) in (&data.spatial, &data.bounding, &*data.entities).join() {

                if powerup.faction == bounding.faction
                    && entity_a != entity_b
                    && powerup.radius + bounding.radius > spatial_a.position.distance(&spatial_b.position) {

                    if let Some(shooter) = data.shooter.get_mut(entity_b) {
                        shooter.spawner = powerup.spawner;
                        data.entities.delete(entity_a).unwrap();
                        //todo explosion
                    }
                }
            }
		}

	}
}
