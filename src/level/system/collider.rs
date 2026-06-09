use crate::prelude::*;
use hecs;
use crate::def::{FactionId, SpawnerId};
use crate::level::component;
use crate::level::WorldState;

pub fn run(world: &mut hecs::World, ws: &WorldState, cmd: &mut hecs::CommandBuffer) {
    // Collect entity data to avoid holding world borrows during mutation
    let entities: Vec<(hecs::Entity, Vec2, f32, FactionId, Option<SpawnerId>)> = world
        .query::<(&component::Spatial, &component::Bounding, &component::Hitpoints, Option<&component::Explodes>)>()
        .iter()
        .map(|(e, (s, b, _, exp))| (e, s.position, b.radius, b.faction, exp.map(|x| x.spawner)))
        .collect();

    let mut collisions: Vec<(hecs::Entity, hecs::Entity, Vec2, Vec2)> = Vec::new();

    for (i, &(ea, pos_a, rad_a, fac_a, _)) in entities.iter().enumerate() {
        for &(eb, pos_b, rad_b, fac_b, _) in entities[i + 1..].iter() {
            if fac_a != fac_b && rad_a + rad_b > pos_a.distance(&pos_b) {
                collisions.push((ea, eb, pos_a, pos_b));
            }
        }
    }

    for (ea, eb, pos_a, pos_b) in collisions {
        let a = match world.get::<&component::Hitpoints>(ea) {
            Ok(hp) => hp.0,
            Err(_) => continue,
        };
        let b = match world.get::<&component::Hitpoints>(eb) {
            Ok(hp) => hp.0,
            Err(_) => continue,
        };

        let damage = min(a, b);

        let exp_a = entities.iter().find(|e| e.0 == ea).and_then(|e| e.4);
        let exp_b = entities.iter().find(|e| e.0 == eb).and_then(|e| e.4);

        if a <= b {
            if let Some(spawner_id) = exp_a {
                ws.spawner(cmd, spawner_id, Angle(0.), Some(pos_a), None, None);
            }
        } else {
            if let Some(spawner_id) = exp_b {
                ws.spawner(cmd, spawner_id, Angle(0.), Some(pos_b), None, None);
            }
        }

        if let Ok(mut hp) = world.get::<&mut component::Hitpoints>(ea) { hp.0 -= damage; }
        if let Ok(mut hp) = world.get::<&mut component::Hitpoints>(eb) { hp.0 -= damage; }
    }
}
