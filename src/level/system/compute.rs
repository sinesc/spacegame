use prelude::*;
use rodio;
use specs;
use level::component;
use level::WorldState;

/**
 * Compute system
 * 
 * This system handles game controlled entities.
 */
pub struct Compute {
}

impl Compute {
    pub fn new() -> Self {
        Compute {
        }
    }
}


#[derive(SystemData)]
pub struct ComputeData<'a> {
    world_state: specs::Fetch<'a, WorldState>,
    controlled: specs::ReadStorage<'a, component::Controlled>,
    computed: specs::WriteStorage<'a, component::Computed>,
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

impl<'a> specs::System<'a> for Compute {
    type SystemData = ComputeData<'a>;

    fn run(&mut self, mut data: ComputeData) {
		use specs::Join;
        use std::f32::consts::PI;

        let mut projectiles = Vec::new();
        let mut target_pos = Vec2(-1., -1.);

        for (controlled, spatial) in (&data.controlled, &data.spatial).join() {
            if controlled.input_id == 1 {
                target_pos = spatial.position;
            }
        }

        if target_pos.0 == -1. {
            for (computed, spatial) in (&data.computed, &data.spatial).join() {
                target_pos = spatial.position;
            }
        }

		for (computed, spatial, inertial, shooter) in (&mut data.computed, &mut data.spatial, &mut data.inertial, &mut data.shooter).join() {

            let target_angle = (target_pos - spatial.position).to_angle();
            spatial.angle.align_with(&target_angle);

            // gradually approach the angle computed from flight direction
            approach(&mut spatial.angle, &target_angle, 1.0 * data.world_state.delta);

            // and reduce angular velocity of manual rotation to 0
            //approach(&mut inertial.av_current, &0.0, inertial.av_trans * data.world_state.delta);

            // lean into rotation direction
            //approach(&mut spatial.lean, &0.0, data.world_state.delta);

            inertial.v_fraction = spatial.angle.to_vec2();

            // shoot ?

            if shooter.interval.elapsed(data.world_state.age) {
                //inertial.v_fraction -= spatial.angle.to_vec2() * 0.001 / data.world_state.delta;
                projectiles.push((spatial.position, spatial.angle));
                rodio::play_raw(&data.world_state.inf.audio, data.world_state.inf.pew.samples());
            }
		}

        let mut spawn = |origin: Vec2, angle: Angle| {
            let shot = data.entities.create();
            data.spatial.insert(shot, component::Spatial::new(origin, angle));
            data.visual.insert(shot, component::Visual::new(Some(data.world_state.inf.layer["effects"].clone()), None, data.world_state.inf.sprite.clone(), Color::RED, 1.0, 30, 0.2));
            data.inertial.insert(shot, component::Inertial::new(Vec2(1133.0, 1133.0), Vec2::from_angle(angle), 1.0));
            data.lifetime.insert(shot, component::Lifetime(data.world_state.age + 1.0));
            data.fading.insert(shot, component::Fading::new(data.world_state.age + 0.5, data.world_state.age + 1.0));
            data.bounding.insert(shot, component::Bounding::new(5.0, 0));
            data.hitpoints.insert(shot, component::Hitpoints::new(50.0));
        };

        for (mut position, angle) in projectiles {
            spawn(position, angle);
        }

	}
}
