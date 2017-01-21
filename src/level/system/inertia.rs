#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use specs;
use radiant_rs::Vec2;
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
            spatial.position += inertial.v_current;

            // compute angle and left/right leaning

            /*if inertial.v_current.len() > 0.01 {
                let old_angle = spatial.angle;
                spatial.angle = inertial.v_current.to_radians();
                // ignore moment angle passes the full circle
                if (old_angle - spatial.angle).abs() < 3.0 {
                    let current_lean = 5.0 * (spatial.angle - old_angle);
                    // average over past 10 lean samples
                    spatial.lean = (9.0*spatial.lean + current_lean) / 10.0;
                }
            }*/
		}
	}
}
