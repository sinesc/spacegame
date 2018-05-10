#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use prelude::*;
use specs;
use specs::Join;
use specs::SystemData;
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

            // compute max inertial angular velocity 

            let v_factor = (inertial.v_current.len() / inertial.v_max.len()).powi(2);
            let av_max = lerp(&inertial.av_max_v0, &inertial.av_max_vmax, v_factor) * delta;

            // compute inertial velocity

            let v_trans = lerp(&inertial.trans_rest, &inertial.trans_motion, inertial.v_fraction.len());
            let v_current_target = lerp(&inertial.v_current, &(inertial.v_max * inertial.v_fraction), v_trans * delta);

            // limit change in direction of velocity vector to max angular velocity

            let old_angle = inertial.v_current.to_angle();
            let mut target_angle = v_current_target.to_angle();
            target_angle.align_with(&old_angle);

            let mut av_current = (target_angle - old_angle).to_radians();

            inertial.v_current = if av_current.abs() > av_max {
                v_current_target.len() * (old_angle + Angle(av_max) * av_current.signum()).to_vec2()
            } else {
                v_current_target
            };

            // lean into rotation direction

            let current_lean = clamp(av_current / av_max * (0.3 + v_factor), -1., 1.);
            approach(&mut spatial.lean, &current_lean, inertial.trans_lean * data.world_state.delta);

            // update spatial position

            spatial.position += inertial.v_current * delta;

            if inertial.motion_type != component::InertialMotionType::Detached {
                spatial.angle = inertial.v_current.to_angle();
            }





            // todo: edge reflection just for fun right now
            if let Some(outbound) = spatial.position.outbound(((0.0, 0.0), (1920.0, 1080.0))) {

                let edge_normal = -outbound.normalize();
                let reflection = inertial.v_current - 2.0 * (inertial.v_current.dot(&edge_normal)) * edge_normal;

                spatial.position -= outbound;
                inertial.v_current = reflection;
                inertial.v_fraction = reflection.normalize() * inertial.v_fraction.len();

                if inertial.motion_type != component::InertialMotionType::Detached {
                    spatial.angle = inertial.v_fraction.to_angle();
                }
            }
		}
	}
}

// TODO: move to radiant-utils
pub fn clamp<T>(a: T, min: T, max: T) -> T where T: PartialOrd {
    if a.lt(&min) { min } else if a.gt(&max) { max } else { a }
}
