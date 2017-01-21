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

fn input(input: &Input, input_id: u32) -> (f32, f32, bool) {
    use radiant_rs::InputId::*;
    let (up, down, left, right, fire) = if input_id == 1 {
        (CursorUp, CursorDown, CursorLeft, CursorRight, RControl)
    } else {
        (W, S, A, D, LControl)
    };
    let mut vertical = 0.0;
    let mut horizontal = 0.0;
    if input.down(up) { vertical -= 1.0 }
    if input.down(down) { vertical += 1.0 }
    if input.down(left) { horizontal -= 1.0 }
    if input.down(right) { horizontal += 1.0 }
    (horizontal, vertical, input.down(fire))
}

impl<'a> specs::System<WorldState> for Control {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (controlleds, mut spatials, mut inertials, mut visuals) = arg.fetch(|w| (
            w.read::<component::Controlled>(),
            w.write::<component::Spatial>(),
            w.write::<component::Inertial>(),
            w.write::<component::Visual>()
        ));

        let mut spawn = Vec::new();

		for (controlled, spatial, mut inertial) in (&controlleds, &spatials, &mut inertials).iter() {

            let (horizontal, vertical, shoot) = input(&state.inf.input, controlled.input_id);

            inertial.v_fraction = Vec2(horizontal, vertical);

            if shoot {
                spawn.push((spatial.position, spatial.angle));
            }
		}

        for (position, angle) in spawn {
            let shot = arg.create();
            spatials.insert(shot, component::Spatial::new(position + Vec2::from_radians(angle) * 40.0, angle));
            visuals.insert(shot, component::Visual::new(state.inf.layer, state.inf.sprite, 30));
            inertials.insert(shot, component::Inertial::new(Vec2(10.0, 10.0), Vec2::from_radians(angle), 4.0, 1.0));
        }
	}
}
