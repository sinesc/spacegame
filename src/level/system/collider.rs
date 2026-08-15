use crate::prelude::*;
use hecs;
use crate::def::FactionId;
use crate::level::component;

pub fn run(world: &mut hecs::World) {
    // Collect entity data to avoid holding world borrows during mutation
    let entities: Vec<(hecs::Entity, Vec2, f32, FactionId)> = world
        .query::<(&component::Spatial, &component::Bounding, &component::Hitpoints)>()
        .iter()
        .map(|(e, (s, b, _))| (e, s.position, b.radius, b.faction))
        .collect();

    let mut collisions: Vec<(hecs::Entity, hecs::Entity)> = Vec::new();

    for (i, &(ea, pos_a, rad_a, fac_a)) in entities.iter().enumerate() {
        for &(eb, pos_b, rad_b, fac_b) in entities[i + 1..].iter() {
            if fac_a != fac_b && rad_a + rad_b > pos_a.distance(&pos_b) {
                collisions.push((ea, eb));
            }
        }
    }

    for (ea, eb) in collisions {
        let a = match world.get::<&component::Hitpoints>(ea) {
            Ok(hp) => hp.0,
            Err(_) => continue,
        };
        let b = match world.get::<&component::Hitpoints>(eb) {
            Ok(hp) => hp.0,
            Err(_) => continue,
        };

        let damage = min(a, b);

        if let Ok(mut hp) = world.get::<&mut component::Hitpoints>(ea) { hp.0 -= damage; }
        if let Ok(mut hp) = world.get::<&mut component::Hitpoints>(eb) { hp.0 -= damage; }
    }
}
