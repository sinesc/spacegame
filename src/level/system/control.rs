use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;

pub struct Control {
    input: Input,
}

impl Control {
    pub fn new(input: &Input) -> Self {
        Control {
            input: input.clone(),
        }
    }
}

impl<'a> specs::System<WorldState> for Control {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (controlleds, mut spatials, mut inertials) = arg.fetch(|w|
			(w.read::<component::Controlled>(), w.write::<component::Spatial>(), w.write::<component::Inertial>())
		);

		for (controlled, mut spatial, mut inertial) in (&controlleds, &mut spatials, &mut inertials).iter() {

            let mut vertical = 0.0;
            let mut horizontal = 0.0;

            for input_id in self.input.iter().down() {
                match input_id {
                    InputId::CursorUp => vertical -= 1.0,
                    InputId::CursorDown => vertical += 1.0,
                    InputId::CursorLeft => horizontal -= 1.0,
                    InputId::CursorRight => horizontal += 1.0,
                    InputId::Mouse1 => horizontal += 5.0,
                    _ => {}
                }
            }

            let trans_current = Vec2(
                if horizontal != 0.0 { inertial.trans_motion } else { inertial.trans_rest },
                if vertical != 0.0 { inertial.trans_motion } else { inertial.trans_rest },
            );

            let v_target = Vec2(
                inertial.v_max.0 * horizontal,
                inertial.v_max.1 * vertical,
            );

            inertial.v_current = inertial.v_current * (Vec2(1.0, 1.0) - state.delta * trans_current) + (v_target * (state.delta * trans_current));

            spatial.pos += inertial.v_current;
		}
	}
}
