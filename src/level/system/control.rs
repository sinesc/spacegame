use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;
use radiant_rs::scene::*;

pub struct Control {
}

impl Control {
    pub fn new() -> Self {
        Control {
        }
    }
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

        let mut test = false;

		for (controlled, mut spatial, mut inertial) in (&controlleds, &mut spatials, &mut inertials).iter() {

            let mut vertical = 0.0;
            let mut horizontal = 0.0;

            for input_id in state.inf.input.iter().down() {
                match input_id {
                    InputId::CursorUp => vertical -= 1.0,
                    InputId::CursorDown => vertical += 1.0,
                    InputId::CursorLeft => horizontal -= 1.0,
                    InputId::CursorRight => horizontal += 1.0,
                    InputId::Mouse1 => {
                        test = true;
                    },
                    _ => {}
                }
            }

            inertial.v_fraction = Vec2(horizontal, vertical);
		}

        if (test) {
            let shot = arg.create();
            spatials.insert(shot, component::Spatial::new(Vec2(300.0, 220.0), 0.0));
            visuals.insert(shot, component::Visual::new(state.inf.layer, state.inf.sprite));
            inertials.insert(shot, component::Inertial::new(Vec2(10.0, 8.0), Vec2(1.0, 1.0), 4.0, 1.0));
        }
	}
}
