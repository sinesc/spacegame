use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;
use std::f32::consts::PI;

pub struct Control {
}

impl Control {
    pub fn new() -> Self {
        Control {
        }
    }
}

fn input(input: &Input, input_id: u32) -> (Vec2, bool, bool) {
    use radiant_rs::InputId::*;
    let (up, down, left, right, fire, rotate) = if input_id == 1 {
        (CursorUp, CursorDown, CursorLeft, CursorRight, RControl, RShift)
    } else {
        (W, S, A, D, LControl, LShift)
    };
    let mut v_fraction = Vec2(0.0, 0.0);
    let rotate = input.down(rotate);
    if input.down(up) { v_fraction.1 -= 1.0 }
    if input.down(down) { v_fraction.1 += 1.0 }
    if input.down(left) { v_fraction.0 -= 1.0 }
    if input.down(right) { v_fraction.0 += 1.0 }
    (v_fraction, input.down(fire), rotate)
}

fn approach(value: &mut f32, target_value: f32, fraction: f32) {
    *value = ((fraction - 1.0) * *value + target_value) / fraction;
}

impl<'a> specs::System<WorldState> for Control {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (mut controlleds, mut spatials, mut inertials, mut visuals, mut lifetimes) = arg.fetch(|w| (
            w.write::<component::Controlled>(),
            w.write::<component::Spatial>(),
            w.write::<component::Inertial>(),
            w.write::<component::Visual>(),
            w.write::<component::Lifetime>()
        ));

        let mut spawn = Vec::new();

		for (mut controlled, mut spatial, mut inertial) in (&mut controlleds, &mut spatials, &mut inertials).iter() {

            let (v_fraction, shoot, rotate) = input(&state.inf.input, controlled.input_id);

            // set v_fraction for Inertia

            inertial.v_fraction = if rotate { Vec2(0.0, 0.0) } else { v_fraction };

            // compute target angle

            let new_angle = inertial.v_current.to_radians();

            if spatial.angle.abs() > PI {
                // fix angles outside of +/- PI
                spatial.angle -= spatial.angle.signum() * 2.0 * PI;
            }

            if (spatial.angle.abs() > 0.5 * PI) & (new_angle.abs() > 0.5 * PI) & (spatial.angle.signum() != new_angle.signum()) {
                // fix rotation step from + <-> - PI by temporarily representing the angle smaller +/- PI with its greater +/- PI equivalent
                spatial.angle = if new_angle.signum() == -1.0 { -PI - (PI - spatial.angle) } else { PI - (-PI - spatial.angle)};
            }

            // backup old angle for comparison agains new angle

            let old_angle = spatial.angle;

            if rotate {

                // acellerating angular velocity (av)
                let av_target = controlled.av_max * v_fraction.1;
                controlled.av_current = controlled.av_current * (1.0 - state.delta * controlled.av_trans) + (av_target * (state.delta * controlled.av_trans));

                // change angle based on player input (rotate ship)
                spatial.angle += controlled.av_current * state.delta;

            } else if v_fraction.len() > 0.0 {

                // gradually approach the angle computed from flight direction
                approach(&mut spatial.angle, new_angle, 5.0);
                controlled.av_current = controlled.av_current * (1.0 - state.delta * controlled.av_trans) + (0.0 * (state.delta * controlled.av_trans));
            }

            // lean into rotation direction

            let current_lean = 5.0 * (spatial.angle - old_angle);
            approach(&mut spatial.lean, current_lean, 10.0);

            // shoot ?

            if shoot {
                spawn.push((spatial.position, spatial.angle));
            }
		}

        for (position, angle) in spawn {
            let shot = arg.create();
            spatials.insert(shot, component::Spatial::new(position + Vec2::from_radians(angle) * 40.0, angle));
            visuals.insert(shot, component::Visual::new(state.inf.layer, state.inf.sprite, 30));
            inertials.insert(shot, component::Inertial::new(Vec2(15.0, 15.0), Vec2::from_radians(angle), 4.0, 1.0));
            lifetimes.insert(shot, component::Lifetime(state.age + 1.5));
        }
	}
}
