#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use specs;
use level::component;
use level::WorldState;

pub struct Inertia;

impl<'a> specs::System<WorldState> for Inertia {
	fn run(&mut self, arg: specs::RunArg, state: WorldState) {
		use specs::Join;
		let (mut space, inertia) = arg.fetch(|w| (w.write::<component::Spatial>(), w.read::<component::Inertial>()));
		for (s, i) in (&mut space, &inertia).iter() {
			s.pos = s.pos + i.velocity * state.delta;
            //s.orient = s.orient + i.angular_velocity * state.delta;
		}
        //println!("inertia");
	}
}
