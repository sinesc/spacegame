#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use specs;
use level::component;
use level::WorldState;

pub struct Inertia;

impl Inertia {
    pub fn new() -> Self {
        Inertia { }
    }
}

impl specs::System<WorldState> for Inertia {

	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;

		let (mut spatials, intertials) = arg.fetch(|w|
            (w.write::<component::Spatial>(), w.read::<component::Inertial>())
        );

		for (s, i) in (&mut spatials, &intertials).iter() {
			//s.pos = s.pos + i.velocity * state.delta;
            //s.orient = s.orient + i.angular_velocity * state.delta;
		}
        //println!("inertia");
	}
}
