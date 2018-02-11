use prelude::*;
use specs;
use level::component;
use level::WorldState;

/**
 * Control system
 * 
 * This system handles player controlled entities.
 */
pub struct Control {
}

impl Control {
    pub fn new() -> Self {
        Control {
        }
    }
}

fn input(input: &Input, input_id: u32) -> (Vec2, bool, bool) {
    use InputId::*;
    let (up, down, left, right, fire, strafe) = if input_id == 1 {
        (CursorUp, CursorDown, CursorLeft, CursorRight, RControl, RShift)
    } else {
        (W, S, A, D, LControl, LShift)
    };
    let mut v_fraction = Vec2(0.0, 0.0);
    if input.down(up) { v_fraction.1 -= 1.0 }
    if input.down(down) { v_fraction.1 += 1.0 }
    if input.down(left) { v_fraction.0 -= 1.0 }
    if input.down(right) { v_fraction.0 += 1.0 }
    v_fraction = v_fraction.normalize();   
    v_fraction.0 += input.mouse_delta().0 as f32 / 5.;
    v_fraction.1 += input.mouse_delta().1 as f32 / 5.;
    (v_fraction, input.down(fire) || input.down(Mouse1), input.down(strafe))
}

#[derive(SystemData)]
pub struct ControlData<'a> {
    world_state: specs::Fetch<'a, WorldState>,
    controlled: specs::WriteStorage<'a, component::Controlled>,
    spatial: specs::WriteStorage<'a, component::Spatial>,
    inertial: specs::WriteStorage<'a, component::Inertial>,
    visual: specs::WriteStorage<'a, component::Visual>,
    lifetime: specs::WriteStorage<'a, component::Lifetime>,
    fading: specs::WriteStorage<'a, component::Fading>,
    bounding: specs::WriteStorage<'a, component::Bounding>,
    hitpoints: specs::WriteStorage<'a, component::Hitpoints>,
    shooter: specs::WriteStorage<'a, component::Shooter>,
    entities: specs::Entities<'a>,
    lazy: specs::Fetch<'a, specs::LazyUpdate>,
}

impl<'a> specs::System<'a> for Control {
    type SystemData = ControlData<'a>;

    fn run(&mut self, mut data: ControlData) {
		use specs::Join;
        use std::f32::consts::PI;

        let mut projectiles = Vec::new();

		for (controlled, spatial, inertial, shooter) in (&mut data.controlled, &mut data.spatial, &mut data.inertial, &mut data.shooter).join() {

            let (v_fraction, shoot, strafe) = input(&data.world_state.inf.input, controlled.input_id);

            if strafe {

                // lean into strafe direction
                let current_lean = (inertial.v_current.to_angle() - spatial.angle).to_radians().sin() * v_fraction.len();
                approach(&mut spatial.lean, &current_lean, 10.0 * data.world_state.delta);

            } else if !strafe {

                let target_angle = inertial.v_current.to_angle();
                spatial.angle.align_with(&target_angle);
                let old_angle = spatial.angle;
                let factor = 10.0 * min(1.0, inertial.v_current.len());

                // gradually approach the angle computed from flight direction
                approach(&mut spatial.angle, &target_angle, factor * data.world_state.delta);

                // and reduce angular velocity of manual rotation to 0
                approach(&mut controlled.av_current, &0.0, controlled.av_trans * data.world_state.delta);

                // lean into rotation direction

                let current_lean = (spatial.angle - old_angle).to_radians() / data.world_state.delta / PI;
                approach(&mut spatial.lean, &current_lean, factor * data.world_state.delta);
            }

            inertial.v_fraction = v_fraction;

            // shoot ?

            if shoot && shooter.interval.elapsed(data.world_state.age) {
                //inertial.v_fraction -= spatial.angle.to_vec2() * 0.001 / data.world_state.delta;
                projectiles.push((spatial.position, spatial.angle));

                /*let dir = spatial.angle.to_vec2();
                let pos = spatial.position - (dir * 10.0);
                component::Shooter::shoot(data.lazy, pos + (dir.right() * 30.0), spatial.angle);
                component::Shooter::shoot(data.lazy, pos + (dir.left() * 30.0), spatial.angle);*/
            }
		}

        let mut spawn = |origin: Vec2, angle: Angle| {
            let shot = data.entities.create();
            data.spatial.insert(shot, component::Spatial::new(origin, angle));
            data.visual.insert(shot, component::Visual::new(Some(data.world_state.inf.layer["effects"].clone()), None, data.world_state.inf.sprite.clone(), Color::WHITE, 1.0, 30, 0.2));
            data.inertial.insert(shot, component::Inertial::new(Vec2(1133.0, 1133.0), Vec2::from_angle(angle), 4.0, 1.0, false));
            data.lifetime.insert(shot, component::Lifetime(data.world_state.age + 1.0));
            data.fading.insert(shot, component::Fading::new(data.world_state.age + 0.5, data.world_state.age + 1.0));
            data.bounding.insert(shot, component::Bounding::new(5.0, 1));
            data.hitpoints.insert(shot, component::Hitpoints::new(5.0));
        };

        for (mut position, angle) in projectiles {
            let dir = angle.to_vec2();
            position -= dir * 10.0;
            spawn(position + (dir.right() * 30.0), angle);
            spawn(position + (dir.left() * 30.0), angle);
        }

	}
}
