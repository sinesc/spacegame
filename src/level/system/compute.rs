use prelude::*;
use specs;
use level::component;
use level::WorldState;

/**
 * Compute system
 *
 * This system handles game controlled entities.
 */
pub struct Compute;

#[derive(SystemData)]
pub struct ComputeData<'a> {
    world_state: specs::ReadExpect<'a, WorldState>,
    controlled: specs::ReadStorage<'a, component::Controlled>,
    computed: specs::ReadStorage<'a, component::Computed>,
    spatial: specs::ReadStorage<'a, component::Spatial>,
    inertial: specs::WriteStorage<'a, component::Inertial>,
    bounding: specs::ReadStorage<'a, component::Bounding>,
    shooter: specs::WriteStorage<'a, component::Shooter>,
    entities: specs::Entities<'a>,
    lazy: specs::Read<'a, specs::LazyUpdate>,
}

impl<'a> specs::System<'a> for Compute {
    type SystemData = ComputeData<'a>;

    fn run(&mut self, mut data: ComputeData) {
		use specs::Join;

        let mut projectiles = Vec::new();
        let mut target_pos = Vec2(-1., -1.);
        let age = data.world_state.age;

        for (controlled, spatial) in (&data.controlled, &data.spatial).join() {
            if controlled.input_id == 1 {
                target_pos = spatial.position;
            }
        }

        if target_pos.0 == -1. {
            for (_, spatial) in (&data.computed, &data.spatial).join() {
                target_pos = spatial.position;
            }
        }

		for (_, spatial, inertial, shooter, bounding) in (&data.computed, &data.spatial, &mut data.inertial, &mut data.shooter, &data.bounding).join() {

            // approach position 250px offset to the right (direction normal) of the player

            let right = (target_pos - spatial.position).right().normalize() * 1000.;
            let left = (target_pos - spatial.position).left().normalize() * 1000.;
            let angle = Angle::from(inertial.v_fraction);

            let offset = if angle.diff(Angle::from(right)).to_radians().abs() > angle.diff(Angle::from(left)).to_radians().abs() {
                left
            } else {
                right
            };
            inertial.v_fraction = ((target_pos + offset) - spatial.position).normalize();

            // compute angle between direct vector to player and vector to offset position

            let mut a1 = Angle::from(target_pos - spatial.position);
            let a2 = Angle::from(inertial.v_fraction);
            //a1.align_with(&a2);
            let diff = a2 - a1;

            // shoot towards the offset direction (still obeying rotation limits)

            if shooter.interval.elapsed(age) {
                projectiles.push((spatial.position, spatial.angle - diff, bounding.faction, shooter.spawner));
            }
		}

        for (position, angle, faction, spawner_id) in projectiles {
            data.world_state.spawner(&data.lazy, &data.entities, spawner_id, angle, Some(position), Some(angle), Some(faction));
        }
	}
}
