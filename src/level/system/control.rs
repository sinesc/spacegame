use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;

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


impl<'a> specs::System<WorldState> for Control {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;
        use std::f32::consts::PI;

		let (mut controlleds, mut spatials, mut inertials, mut visuals, mut lifetimes, mut shooters) = arg.fetch(|w| (
            w.write::<component::Controlled>(),
            w.write::<component::Spatial>(),
            w.write::<component::Inertial>(),
            w.write::<component::Visual>(),
            w.write::<component::Lifetime>(),
            w.write::<component::Shooter>()
        ));

        let mut spawn = Vec::new();

		for (mut controlled, mut spatial, mut inertial, mut shooter) in (&mut controlleds, &mut spatials, &mut inertials, &mut shooters).iter() {

            let (v_fraction, shoot, rotate) = input(&state.inf.input, controlled.input_id);

            // set v_fraction for Inertia
            inertial.v_fraction = if rotate { Vec2(0.0, 0.0) } else { v_fraction };

            // compute target angle and align current angle with it (subtraction will then yield the smallest angle between both)
            let new_angle = inertial.v_current.to_angle();
            spatial.angle.align_with(&new_angle);
            let old_angle = spatial.angle;

            if rotate {

                // accelerating angular velocity (av)
                utils::approach(&mut controlled.av_current, controlled.av_max * v_fraction.1, controlled.av_trans * state.delta);

                // change angle based on player input (rotate ship)
                spatial.angle += Angle(controlled.av_current * state.delta);

            } else if v_fraction.len() > 0.0 {

                // gradually approach the angle computed from flight direction
                utils::approach(&mut spatial.angle, new_angle, 10.0 * state.delta);

                // and reduce angular velocity of manual rotation to 0
                utils::approach(&mut controlled.av_current, 0.0, controlled.av_trans * state.delta);
            }

            // lean into rotation direction

            let current_lean = (spatial.angle - old_angle).to_radians() / state.delta / PI;
            utils::approach(&mut spatial.lean, current_lean, 10.0 * state.delta);

            // shoot ?

            if shoot && shooter.next_shot <= state.age {
                shooter.next_shot = state.age + shooter.interval;
                spawn.push((spatial.position, spatial.angle));
            }
		}

        for (position, angle) in spawn {
            let shot = arg.create();
            spatials.insert(shot, component::Spatial::new(position + Vec2::from_angle(angle) * 40.0, angle));
            visuals.insert(shot, component::Visual::new(state.inf.layer, state.inf.sprite, 30));
            inertials.insert(shot, component::Inertial::new(Vec2(1500.0, 1500.0), Vec2::from_angle(angle), 4.0, 1.0));
            lifetimes.insert(shot, component::Lifetime(state.age + 1.5));
        }
	}
}
