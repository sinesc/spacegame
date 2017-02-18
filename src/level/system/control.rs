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


impl<'a> specs::System<WorldState> for Control {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;
        use std::f32::consts::PI;

		let (mut controlleds, mut spatials, mut inertials, mut visuals, mut lifetimes, mut shooters, mut faders, mut boundings) = arg.fetch(|w| (
            w.write::<component::Controlled>(),
            w.write::<component::Spatial>(),
            w.write::<component::Inertial>(),
            w.write::<component::Visual>(),
            w.write::<component::Lifetime>(),
            w.write::<component::Shooter>(),
            w.write::<component::Fading>(),
            w.write::<component::Bounding>()
        ));

        let mut projectiles = Vec::new();

		for (mut controlled, mut spatial, mut inertial, mut shooter) in (&mut controlleds, &mut spatials, &mut inertials, &mut shooters).iter() {

            let (v_fraction, shoot, alternate, strafe) = input(&state.inf.input, controlled.input_id);

            // set v_fraction for Inertia
            inertial.v_fraction = if alternate && strafe == 0.0 {
                Vec2(0.0, 0.0)
            } else if strafe != 0.0 {
                utils::approach(&mut controlled.av_current, &0.0, controlled.av_trans * state.delta); // kill rotation
                let new_vec = spatial.angle.to_vec2() * strafe; // approach new directional vector
                utils::approach(&mut inertial.v_current, &new_vec, state.delta / 100.0);
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
                utils::approach(&mut controlled.av_current, &(controlled.av_max * v_fraction.1), controlled.av_trans * state.delta);

                // change angle based on player input (rotate ship)
                spatial.angle += Angle(controlled.av_current * state.delta);

            } else if strafe == 0.0 && v_fraction.len() > 0.0 {

                // gradually approach the angle computed from flight direction
                utils::approach(&mut spatial.angle, &new_angle, 10.0 * state.delta);

                // and reduce angular velocity of manual rotation to 0
                utils::approach(&mut controlled.av_current, &0.0, controlled.av_trans * state.delta);
            }

            // lean into rotation direction

            let current_lean = (spatial.angle - old_angle).to_radians() / state.delta / PI;
            utils::approach(&mut spatial.lean, &current_lean, 10.0 * state.delta);

            // shoot ?

            if shoot && shooter.interval.elapsed(state.age) {
                inertial.v_fraction -= spatial.angle.to_vec2() * 0.001 / state.delta;
                projectiles.push((spatial.position, spatial.angle));
            }
		}

        let mut spawn = |origin: Point2, angle: Angle| {
            let shot = arg.create();
            spatials.insert(shot, component::Spatial::new(origin, angle, false));
            visuals.insert(shot, component::Visual::new(state.inf.effects.clone(), None, state.inf.sprite.clone(), Color::white(), 30));
            inertials.insert(shot, component::Inertial::new(Vec2(1433.0, 1433.0), Vec2::from_angle(angle), 4.0, 1.0));
            lifetimes.insert(shot, component::Lifetime(state.age + 1.0));
            faders.insert(shot, component::Fading::new(state.age + 0.5, state.age + 1.0));
            boundings.insert(shot, component::Bounding::new(5.0, 1));
        };

        for (mut position, angle) in projectiles {
            let dir = angle.to_vec2();
            position -= dir * 10.0;
            spawn(position + (dir.right() * 30.0), angle);
            spawn(position + (dir.left() * 30.0), angle);
        }
	}
}
