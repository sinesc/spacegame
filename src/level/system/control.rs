use prelude::*;
use rodio;
use specs;
use level::component;
use level::WorldState;

/**
 * Control system
 *
 * This system handles player controlled entities.
 */
pub struct Control {
}

impl Control {
    pub fn new() -> Self {
        Control {
        }
    }
}

fn flatten(x: i32) -> f32 {
    let x = x as f32;
    if x < 0. {
        -((-x).sqrt())
    } else {
        x.sqrt()
    }
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
        // TODO: divider depends on mouse speed. needs to be chose so that moving the mouse reasonably fast equals 1
        v_fraction.0 += flatten(input.mouse_delta().0) / 4.;
        v_fraction.1 += flatten(input.mouse_delta().1) / 4.;
        /*if v_fraction.len() > 1.0 {
            println!("exceeeded! {:?}", v_fraction.len());
            v_fraction /= v_fraction.len();
        }
        println!("len: {:?}", v_fraction.len());*/
        (v_fraction, input.down(fire1) || input.down(fire2), input.down(strafe), input.down(rotate))
    } else {
        (Vec2(0., 0.), false, false, false)
    }
}

#[derive(SystemData)]
pub struct ControlData<'a> {
    world_state: specs::ReadExpect<'a, WorldState>,
    controlled: specs::WriteStorage<'a, component::Controlled>,
    spatial: specs::WriteStorage<'a, component::Spatial>,
    inertial: specs::WriteStorage<'a, component::Inertial>,
    visual: specs::WriteStorage<'a, component::Visual>,
    lifetime: specs::WriteStorage<'a, component::Lifetime>,
    fading: specs::WriteStorage<'a, component::Fading>,
    bounding: specs::WriteStorage<'a, component::Bounding>,
    hitpoints: specs::WriteStorage<'a, component::Hitpoints>,
    shooter: specs::WriteStorage<'a, component::Shooter>,
    entities: specs::Entities<'a>,
    lazy: specs::Read<'a, specs::LazyUpdate>,
}

impl<'a> specs::System<'a> for Control {
    type SystemData = ControlData<'a>;

    fn run(&mut self, mut data: ControlData) {
		use specs::Join;

        let mut projectiles = Vec::new();
        let age = data.world_state.age;

		for (controlled, spatial, inertial, shooter, bounding) in (&mut data.controlled, &mut data.spatial, &mut data.inertial, &mut data.shooter, &mut data.bounding).join() {

            let input_id = if data.world_state.take_input { Some(controlled.input_id) } else { None };
            let (v_fraction, shoot, strafe, rotate) = input(&data.world_state.inf.input, input_id);

            /*if controlled.input_id == 1 {
                data.world_state.inf.font.write(
                    &data.world_state.inf.layer["text"],
                    &format!("Input\nv_fraction: ({:.3} {:.3})\nshoot: {:?}\nstrafe: {:?}\nrotate: {:?}", v_fraction.0, v_fraction.1, shoot, strafe, rotate),
                    (10.0, 300.0),
                    Color::alpha_pm(0.4)
                );
            }*/

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

            // shoot ?

            if shoot && shooter.interval.elapsed(age) {
                projectiles.push((spatial.position, spatial.angle, bounding.faction, shooter.spawner));
            }
		}

        for (position, angle, faction, spawner_id) in projectiles {
            data.world_state.spawner(&data.lazy, &data.entities, spawner_id, angle, Some(position), Some(angle), Some(faction));
        }

	}
}
