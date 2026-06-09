use prelude::*;
use hecs;
use level::component;
use level::WorldState;

pub fn run(world: &mut hecs::World, ws: &WorldState, cmd: &mut hecs::CommandBuffer) {
    use def::{SpawnerId, FactionId};

    // Collect powerup entity data
    let powerups: Vec<(hecs::Entity, Vec2, f32, FactionId, SpawnerId, Option<SpawnerId>)> = world
        .query::<(&component::Spatial, &component::Powerup, Option<&component::Explodes>)>()
        .iter()
        .map(|(e, (s, p, exp))| (e, s.position, p.radius, p.faction, p.spawner, exp.map(|x| x.spawner)))
        .collect();

    // Collect bounding entity data (potential pickup targets)
    let bounders: Vec<(hecs::Entity, Vec2, f32, FactionId)> = world
        .query::<(&component::Spatial, &component::Bounding)>()
        .iter()
        .map(|(e, (s, b))| (e, s.position, b.radius, b.faction))
        .collect();

    let mut pickups: Vec<(hecs::Entity, hecs::Entity, SpawnerId, Option<SpawnerId>, Vec2)> = Vec::new();

    'outer: for &(ea, pos_a, rad_a, fac_a, pw_spawner, exp_spawner) in &powerups {
        for &(eb, pos_b, rad_b, fac_b) in &bounders {
            if fac_a == fac_b && ea != eb && rad_a + rad_b > pos_a.distance(&pos_b) {
                pickups.push((ea, eb, pw_spawner, exp_spawner, pos_a));
                continue 'outer;
            }
        }
    }

    let mut to_despawn = Vec::new();

    for (ea, eb, pw_spawner, exp_spawner, pos_a) in pickups {
        if let Ok(mut shooter) = world.get::<&mut component::Shooter>(eb) {
            shooter.spawner = pw_spawner;
            drop(shooter);
            if let Some(exp_spawner_id) = exp_spawner {
                ws.spawner(cmd, exp_spawner_id, Angle(0.), Some(pos_a), None, None);
            }
            to_despawn.push(ea);
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}
