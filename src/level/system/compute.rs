use prelude::*;
use hecs;
use level::component;
use level::WorldState;

pub fn run(world: &mut hecs::World, ws: &WorldState, cmd: &mut hecs::CommandBuffer) {
    let mut projectiles = Vec::new();
    let mut target_pos = Vec2(-1., -1.);
    let age = ws.age;

    for (_e, (controlled, spatial)) in world.query::<(&component::Controlled, &component::Spatial)>().iter() {
        if controlled.input_id == 1 {
            target_pos = spatial.position;
        }
    }

    if target_pos.0 == -1. {
        for (_e, (_, spatial)) in world.query::<(&component::Computed, &component::Spatial)>().iter() {
            target_pos = spatial.position;
        }
    }

    for (_entity, (_, spatial, inertial, shooter, bounding)) in world.query_mut::<(
        &component::Computed,
        &component::Spatial,
        &mut component::Inertial,
        &mut component::Shooter,
        &component::Bounding,
    )>() {
        let right = (target_pos - spatial.position).right().normalize() * 1000.;
        let left = (target_pos - spatial.position).left().normalize() * 1000.;
        let angle = Angle::from(inertial.v_fraction);

        let offset = if angle.diff(Angle::from(right)).to_radians().abs() > angle.diff(Angle::from(left)).to_radians().abs() {
            left
        } else {
            right
        };
        inertial.v_fraction = ((target_pos + offset) - spatial.position).normalize();

        let a1 = Angle::from(target_pos - spatial.position);
        let a2 = Angle::from(inertial.v_fraction);
        let diff = a2 - a1;

        if shooter.interval.elapsed(age) {
            projectiles.push((spatial.position, spatial.angle - diff, bounding.faction, shooter.spawner));
        }
    }

    for (position, angle, faction, spawner_id) in projectiles {
        ws.spawner(cmd, spawner_id, angle, Some(position), Some(angle), Some(faction));
    }
}
