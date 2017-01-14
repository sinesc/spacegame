use std::sync::Arc;
use specs;
use level::component;
use level::WorldState;
use radiant_rs::*;

pub struct Control {
    input: Input,
}

impl<'a> Control {
    pub fn new(input: &Input) -> Self {
        Control {
            input: input.clone(),
        }
    }
}

impl<'a> specs::System<WorldState> for Control {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (mut spatials, controlleds) = arg.fetch(|w|
			(w.write::<component::Spatial>(), w.read::<component::Controlled>())
		);

		for (mut spatial, controlleds) in (&mut spatials, &controlleds).iter() {
            spatial.pos = Vec2(self.input.mouse_x() as f32, self.input.mouse_y() as f32);
		}
	}
}
