#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use specs;
use specs::Join;
use specs::SystemData;
use radiant_rs::*;
use radiant_rs::math::*;
use level::component;
use level::WorldState;

/**
 * Inertia system
 * 
 * Applies force to entities with an Inertial and Spatial component.
 */
pub struct Inertia;

impl Inertia {
    pub fn new() -> Self {
        Inertia { }
    }
}

#[derive(SystemData)]
pub struct InertiaData<'a> {
    world_state: specs::Fetch<'a, WorldState>,
    spatial: specs::WriteStorage<'a, component::Spatial>,
    inertial: specs::WriteStorage<'a, component::Inertial>,
}

impl<'a> specs::System<'a> for Inertia {
    type SystemData = InertiaData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
		use specs::Join;

        let delta = data.world_state.delta;

        for (spatial, inertial) in (&mut data.spatial, &mut data.inertial).join() {

            // todo: make this scale linearly between trans_rest and trans_motion based on v_fraction?
            let trans_current = Vec2(
                if inertial.v_fraction.0 != 0.0 { inertial.trans_motion } else { inertial.trans_rest },
                if inertial.v_fraction.1 != 0.0 { inertial.trans_motion } else { inertial.trans_rest },
            );

            let v_target = inertial.v_max * inertial.v_fraction;

            inertial.v_current = inertial.v_current * (Vec2(1.0, 1.0) - delta * trans_current) + (v_target * (delta * trans_current));
            spatial.position += inertial.v_current * delta;

            // todo: edge reflection just for fun right now
            if let Some(outbound) = spatial.position.outbound(Rect::new(0.0, 0.0, 1600.0, 900.0)) {

                let edge_normal = -outbound.normalize();
                let reflection = inertial.v_current - 2.0 * (inertial.v_current.dot(&edge_normal)) * edge_normal;

                spatial.position -= outbound;
                inertial.v_current = reflection;
                inertial.v_fraction = reflection.normalize() * inertial.v_fraction.len();

                if !inertial.angle_locked {
                    spatial.angle = inertial.v_fraction.to_angle();
                }
            }
		}
	}
}
