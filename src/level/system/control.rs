use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;
use radiant_rs::math::*;

pub struct Control {
}

impl Control {
    pub fn new() -> Self {
        Control {
        }
    }
}

fn input(input: &Input, input_id: u32) -> (Vec2, bool, bool, f32) {
    use radiant_rs::InputId::*;
    let (up, down, left, right, fire, alternate) = if input_id == 1 {
        (CursorUp, CursorDown, CursorLeft, CursorRight, RControl, RShift)
    } else {
        (W, S, A, D, LControl, LShift)
    };
    let mut v_fraction = Vec2(0.0, 0.0);
    let alternate = input.down(alternate);
    if input.down(up) { v_fraction.1 -= 1.0 }
    if input.down(down) { v_fraction.1 += 1.0 }
    if input.down(left) { v_fraction.0 -= 1.0 }
    if input.down(right) { v_fraction.0 += 1.0 }
    (v_fraction.normalize(), input.down(fire), alternate, if alternate { v_fraction.0 } else { 0.0 })
}

#[derive(SystemData)]
pub struct ControlData<'a> {
    world_state: specs::Fetch<'a, WorldState>,
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
}

impl<'a> specs::System<'a> for Control {
    type SystemData = ControlData<'a>;

    fn run(&mut self, mut data: ControlData) {
		use specs::Join;
        use std::f32::consts::PI;

        let mut projectiles = Vec::new();

		for (mut controlled, mut spatial, mut inertial, mut shooter) in (&mut data.controlled, &mut data.spatial, &mut data.inertial, &mut data.shooter).join() {

            let (v_fraction, shoot, alternate, strafe) = input(&data.world_state.inf.input, controlled.input_id);

            // set v_fraction for Inertia
            inertial.v_fraction = if alternate && strafe == 0.0 {
                Vec2(0.0, 0.0)
            } else if strafe != 0.0 {
                utils::approach(&mut controlled.av_current, &0.0, controlled.av_trans * data.world_state.delta); // kill rotation
                let new_vec = spatial.angle.to_vec2() * strafe; // approach new directional vector
                utils::approach(&mut inertial.v_current, &new_vec, data.world_state.delta / 100.0);
                new_vec
            } else {
                v_fraction
            };

            // compute target angle and align current angle with it (subtraction will then yield the smallest angle between both)
            let new_angle = inertial.v_current.to_angle();
            spatial.angle.align_with(&new_angle);
            let old_angle = spatial.angle;

            if alternate {

                // accelerating angular velocity (av)
                utils::approach(&mut controlled.av_current, &(controlled.av_max * v_fraction.1), controlled.av_trans * data.world_state.delta);

                // change angle based on player input (rotate ship)
                spatial.angle += Angle(controlled.av_current * data.world_state.delta);

            } else if strafe == 0.0 && v_fraction.len() > 0.0 {

                // gradually approach the angle computed from flight direction
                utils::approach(&mut spatial.angle, &new_angle, 10.0 * data.world_state.delta);

                // and reduce angular velocity of manual rotation to 0
                utils::approach(&mut controlled.av_current, &0.0, controlled.av_trans * data.world_state.delta);
            }

            // lean into rotation direction

            let current_lean = (spatial.angle - old_angle).to_radians() / data.world_state.delta / PI;
            utils::approach(&mut spatial.lean, &current_lean, 10.0 * data.world_state.delta);

            // shoot ?

            if shoot && shooter.interval.elapsed(data.world_state.age) {
                inertial.v_fraction -= spatial.angle.to_vec2() * 0.001 / data.world_state.delta;
                projectiles.push((spatial.position, spatial.angle));
            }
		}

        let mut spawn = |origin: Vec2, angle: Angle| {
            let shot = data.entities.create();
            data.spatial.insert(shot, component::Spatial::new(origin, angle, false));
            data.visual.insert(shot, component::Visual::new(Some(data.world_state.inf.effects.clone()), None, data.world_state.inf.sprite.clone(), Color::white(), 30, 0.2));
            data.inertial.insert(shot, component::Inertial::new(Vec2(1433.0, 1433.0), Vec2::from_angle(angle), 4.0, 1.0));
            data.lifetime.insert(shot, component::Lifetime(data.world_state.age + 1.0));
            data.fading.insert(shot, component::Fading::new(data.world_state.age + 0.5, data.world_state.age + 1.0));
            data.bounding.insert(shot, component::Bounding::new(5.0, 1));
            data.hitpoints.insert(shot, component::Hitpoints::new(10.0));
        };

        for (mut position, angle) in projectiles {
            let dir = angle.to_vec2();
            position -= dir * 10.0;
            spawn(position + (dir.right() * 30.0), angle);
            spawn(position + (dir.left() * 30.0), angle);
        }
	}
}
