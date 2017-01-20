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

        let mut spawn = Vec::new();

		for (controlled, spatial, mut inertial) in (&controlleds, &spatials, &mut inertials).iter() {

            let mut vertical = 0.0;
            let mut horizontal = 0.0;
            let mut shoot = false;

            for input_id in state.inf.input.iter().down() {
                match input_id {
                    InputId::CursorUp => vertical -= 1.0,
                    InputId::CursorDown => vertical += 1.0,
                    InputId::CursorLeft => horizontal -= 1.0,
                    InputId::CursorRight => horizontal += 1.0,
                    InputId::Mouse1 => shoot = true,
                    _ => {}
                }
            }

            inertial.v_fraction = Vec2(horizontal, vertical);

            if shoot {
                spawn.push((spatial.position, spatial.angle));
            }
		}

        for (position, angle) in spawn {
            let shot = arg.create();
            spatials.insert(shot, component::Spatial::new(position, angle));
            visuals.insert(shot, component::Visual::new(state.inf.layer, state.inf.sprite, 30));
            inertials.insert(shot, component::Inertial::new(Vec2(1.0, 1.0), Vec2::from_rad(angle), 4.0, 1.0));
        }
	}
}
