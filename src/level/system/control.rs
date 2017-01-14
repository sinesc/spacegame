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

            // !todo ugly

            let left = !!self.input.cursor_left();
            let right = !!self.input.cursor_right();
            let up = !!self.input.cursor_up();
            let down = !!self.input.cursor_down();

            let trans_current = if left ^ right | up ^ down { inertial.trans_motion } else { inertial.trans_rest };
            let v_target = Vec2(
                if left & !right { -inertial.v_max.0 } else if right & !left { inertial.v_max.0 } else { 0.0 },
                if up & !down { -inertial.v_max.1 } else if down & !up { inertial.v_max.1 } else { 0.0 }
            );

            inertial.v_current = inertial.v_current * (1.0 - state.delta * trans_current) + v_target * (state.delta * trans_current);

            spatial.pos += inertial.v_current;
		}
	}
}
