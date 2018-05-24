use prelude::*;
use specs;
use level::component;
use level::WorldState;

/**
 * Inertia system
 *
 * Applies force to entities with an Inertial and Spatial component.
 */
pub struct Inertia;

#[derive(SystemData)]
pub struct InertiaData<'a> {
    world_state: specs::ReadExpect<'a, WorldState>,
    spatial: specs::WriteStorage<'a, component::Spatial>,
    inertial: specs::WriteStorage<'a, component::Inertial>,
}

impl<'a> specs::System<'a> for Inertia {
    type SystemData = InertiaData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
		use specs::Join;

        let delta = data.world_state.delta;

        for (spatial, inertial) in (&mut data.spatial, &mut data.inertial).join() {

            // trans-factor for current velocity

            let v_trans = lerp(&inertial.trans_rest, &inertial.trans_motion, inertial.v_fraction.len());

            if inertial.motion_type == component::InertialMotionType::FollowVector {

                // compute max inertial angular velocity

                let v_factor = (inertial.v_current.len() / inertial.v_max.len()).powi(2);
                let av_max = lerp(&inertial.av_max_v0, &inertial.av_max_vmax, v_factor) * delta;

                // limit change in direction of velocity vector to max angular velocity

                let v_current_target = lerp(&inertial.v_current, &(inertial.v_max * inertial.v_fraction), v_trans * delta);
                let old_angle = Angle::from(inertial.v_current);
                let mut target_angle = Angle::from(v_current_target);
                target_angle.align_with(old_angle);

                let mut av_current = (target_angle - old_angle).to_radians();

                inertial.v_current = if av_current.abs() > av_max {
                    v_current_target.len() * Vec2::from(old_angle + Angle(av_max) * av_current.signum())
                } else {
                    v_current_target
                };

                // lean into rotation direction

                if av_max > 0. {
                    let current_lean = clamp(av_current / av_max * (0.4 + v_factor), -1., 1.);
                    approach(&mut spatial.lean, &current_lean, inertial.trans_lean * delta);
                }

                // update spatial angle

                spatial.angle = Angle::from(inertial.v_current);

            } else if inertial.motion_type == component::InertialMotionType::StrafeVector {

                // approach strafe vector

                approach(&mut inertial.v_current, &(inertial.v_max * inertial.v_fraction), v_trans * delta);

                // lean into strafe direction

                let current_lean = (Angle::from(inertial.v_current) - spatial.angle).to_radians().sin() * inertial.v_fraction.len();
                approach(&mut spatial.lean, &current_lean, 10.0 * data.world_state.delta);

            } else if inertial.motion_type == component::InertialMotionType::Detached {

                // approach full stop

                approach(&mut inertial.v_current, &Vec2(0., 0.), v_trans * delta);

                // compute max inertial angular velocity

                let av_max = inertial.av_max_v0 * delta;

                // limit change in direction of velocity vector to max angular velocity

                let old_angle = spatial.angle;
                let mut target_angle = if inertial.v_fraction.len() > 0. { Angle::from(inertial.v_fraction) } else { spatial.angle };
                target_angle.align_with(old_angle);

                let mut av_current = (target_angle - old_angle).to_radians();

                data.world_state.inf.font.write(
                    &data.world_state.inf.layer["text"],
                    &format!("old_angle: {:.3}\ntarget_angle: {:.3}\nav_current: {:.3}\nav_max: {:.3}",
                    old_angle.to_degrees(), target_angle.to_degrees(), (target_angle - old_angle).to_radians().signum(), Angle(av_max).to_degrees()),
                    (10.0, 500.0),
                    Color::alpha_pm(0.4)
                );

                if av_current.abs() > av_max {
                    spatial.angle += Angle(av_max) * av_current.signum();
                    spatial.angle = spatial.angle.normalize();
                } else {
                    spatial.angle = target_angle;
                }

                // lean into rotation direction

                if av_max > 0. {
                    let v_factor = 1.0;
                    let current_lean = clamp(av_current / av_max * (0.4 + v_factor), -1., 1.);
                    approach(&mut spatial.lean, &current_lean, inertial.trans_lean * delta);
                }

            } else if inertial.motion_type == component::InertialMotionType::Const {

                //inertial.v_current = inertial.v_max * inertial.v_fraction;
            }

            // update spatial position

            spatial.position += inertial.v_current * delta;


            // todo: edge reflection just for fun right now
            if let Some(outbound) = spatial.position.outbound(((0.0, 0.0), (1920.0, 1080.0))) {

                let edge_normal = -outbound.normalize();
                let reflection = inertial.v_current - 2.0 * (inertial.v_current.dot(&edge_normal)) * edge_normal;

                spatial.position -= outbound;
                inertial.v_current = reflection;
                inertial.v_fraction = reflection.normalize() * inertial.v_fraction.len();

                if inertial.motion_type != component::InertialMotionType::Detached {
                    spatial.angle = Angle::from(inertial.v_fraction);
                }
            }
		}
	}
}