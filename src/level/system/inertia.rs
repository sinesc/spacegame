#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use specs;
use radiant_rs::*;
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

		let (mut spatials, mut intertials) = arg.fetch(|w|
            (w.write::<component::Spatial>(), w.write::<component::Inertial>())
        );

		for (spatial, inertial) in (&mut spatials, &mut intertials).iter() {

            // todo: make this scale linearly between trans_rest and trans_motion based on v_fraction?
            let trans_current = Vec2(
                if inertial.v_fraction.0 != 0.0 { inertial.trans_motion } else { inertial.trans_rest },
                if inertial.v_fraction.1 != 0.0 { inertial.trans_motion } else { inertial.trans_rest },
            );

            let v_target = inertial.v_max * inertial.v_fraction;

            inertial.v_current = inertial.v_current * (Vec2(1.0, 1.0) - state.delta * trans_current) + (v_target * (state.delta * trans_current));
            spatial.position += inertial.v_current * state.delta;

            if let Some(outbound) = spatial.position.outbound(Rect::new(0.0, 0.0, 1600.0, 900.0)) {

                let edge_normal = -outbound.normalize();
                let reflection = inertial.v_current - 2.0 * (inertial.v_current.dot(&edge_normal)) * edge_normal;

                spatial.position -= outbound;
                inertial.v_current = reflection;
                inertial.v_fraction = reflection.normalize() * inertial.v_fraction.len();

                // !todo don't want this for player
                spatial.angle = inertial.v_fraction.to_angle();
            }
		}
	}
}
