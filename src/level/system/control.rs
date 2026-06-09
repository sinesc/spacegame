use crate::prelude::*;
use hecs;
use crate::level::component;
use crate::level::WorldState;

fn flatten(x: i32) -> f32 {
    let x = x as f32;
    if x < 0. { -((-x).sqrt()) } else { x.sqrt() }
}

fn input(input: &Input, input_id: Option<u32>) -> (Vec2, bool, bool, bool) {
    use InputId::*;
    if let Some(input_id) = input_id {
        let (up, down, left, right, fire1, fire2, strafe, rotate) = if input_id == 1 {
            (CursorUp, CursorDown, CursorLeft, CursorRight, RMenu, Mouse1, RShift, RControl)
        } else {
            (W, S, A, D, LControl, LControl, LShift, LMenu)
        };
        let mut v_fraction = Vec2(0.0, 0.0);
        if input.down(up) { v_fraction.1 -= 1.0 }
        if input.down(down) { v_fraction.1 += 1.0 }
        if input.down(left) { v_fraction.0 -= 1.0 }
        if input.down(right) { v_fraction.0 += 1.0 }
        v_fraction = v_fraction.normalize();
        v_fraction.0 += flatten(input.mouse_delta().0) / 4.;
        v_fraction.1 += flatten(input.mouse_delta().1) / 4.;
        (v_fraction, input.down(fire1) || input.down(fire2), input.down(strafe), input.down(rotate))
    } else {
        (Vec2(0., 0.), false, false, false)
    }
}

pub fn run(world: &mut hecs::World, ws: &WorldState, cmd: &mut hecs::CommandBuffer) {
    let mut projectiles = Vec::new();
    let age = ws.age;

    for (_entity, (controlled, spatial, inertial, shooter, bounding)) in world.query_mut::<(
        &component::Controlled,
        &component::Spatial,
        &mut component::Inertial,
        &mut component::Shooter,
        &component::Bounding,
    )>() {
        let input_id = if ws.take_input { Some(controlled.input_id) } else { None };
        let (v_fraction, shoot, strafe, rotate) = input(&ws.inf.input, input_id);

        if controlled.input_id == 1 {
            ws.inf.font.write(
                &ws.inf.layer["text"],
                &format!("Input\nv_fraction: ({:.3} {:.3})\nshoot: {:?}\nstrafe: {:?}\nrotate: {:?}",
                    v_fraction.0, v_fraction.1, shoot, strafe, rotate),
                (10.0, 300.0),
                Color::alpha_pm(0.4)
            );
        }

        if strafe {
            inertial.v_fraction = v_fraction;
            inertial.motion_type = component::InertialMotionType::StrafeVector;
        } else if rotate {
            inertial.v_fraction = v_fraction;
            inertial.motion_type = component::InertialMotionType::Detached;
        } else {
            inertial.v_fraction = v_fraction;
            inertial.motion_type = component::InertialMotionType::FollowVector;
        }

        if shoot && shooter.interval.elapsed(age) {
            projectiles.push((spatial.position, spatial.angle, bounding.faction, shooter.spawner));
        }
    }

    for (position, angle, faction, spawner_id) in projectiles {
        ws.spawner(cmd, spawner_id, angle, Some(position), Some(angle), Some(faction));
    }
}
