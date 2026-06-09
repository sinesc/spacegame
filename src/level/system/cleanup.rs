use hecs;
use level::component;
use level::WorldState;

pub fn run(world: &mut hecs::World, ws: &WorldState) {
    let mut to_despawn = Vec::new();

    for (entity, lifetime) in world.query::<&component::Lifetime>().iter() {
        if lifetime.0 < ws.age {
            to_despawn.push(entity);
        }
    }

    for (entity, hitpoints) in world.query::<&component::Hitpoints>().iter() {
        if hitpoints.0 <= 0. {
            to_despawn.push(entity);
        }
    }

    to_despawn.sort_unstable();
    to_despawn.dedup();

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}
