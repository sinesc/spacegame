use specs;
use radiant_rs::*;
use radiant_rs::scene::*;
use std::sync::Arc;

mod component;
mod system;

pub struct Infrastructure {
    scene   : Scene,
    input   : Input,

    sprite: SpriteId,
    layer: LayerId,
}

#[derive(Clone)]
pub struct WorldState {
    delta   : f32,
    inf     : Arc<Infrastructure>,
}

pub struct Level {
    planner : specs::Planner<WorldState>,
    inf     : Arc<Infrastructure>,
}

impl Level {

    pub fn new(input: &Input, context: &RenderContext) -> Level {

        // create world and register components

        let mut world = specs::World::new();
        world.register::<component::Spatial>();
        world.register::<component::Inertial>();
        world.register::<component::Visual>();
        world.register::<component::Controlled>();

        // create a scene and a layer

        let scene = Scene::new(context);
        let base = scene.register_layer(1024, 768);
        let effects = scene.register_layer(1024, 768);
        let hostile = scene.register_sprite_from_file("res/sprite/hostile/mine_red_64x64x15.png");
        let friend = scene.register_sprite_from_file("res/sprite/player/speedy_98x72x30.png");
        let powerup = scene.register_sprite_from_file("res/sprite/powerup/ball_v_32x32x18.jpg");

        let laser = scene.register_sprite_from_file("res/sprite/projectile/bolt_white_60x36x1.jpg");

        scene.op(Op::SetBlendmode(base, blendmodes::ALPHA));
        scene.op(Op::SetBlendmode(effects, blendmodes::LIGHTEN));
        scene.op(Op::Draw(base));
        scene.op(Op::Draw(effects));
        scene.op(Op::Clear(base));
        scene.op(Op::Clear(effects));
        scene.op(Op::RotateModelMatrixAt(base, 1.0, Vec2(0.0, 0.0), 0.1));

        // create test entity

        world.create_now()
            .with(component::Spatial::new(Vec2(330.0, 250.0), 0.0))
            .with(component::Visual::new(base, friend))
            .build();

        world.create_now()
            .with(component::Spatial::new(Vec2(320.0, 240.0), 0.0))
            .with(component::Visual::new(base, hostile))
            .build();

        world.create_now()
            .with(component::Spatial::new(Vec2(330.0, 250.0), 0.0))
            .with(component::Visual::new(effects, powerup))
            .build();

        world.create_now()
            .with(component::Spatial::new(Vec2(300.0, 220.0), 0.0))
            .with(component::Visual::new(base, friend))
            .with(component::Inertial::new(Vec2(10.0, 8.0), Vec2(0.0, 0.0), 4.0, 1.0))
            .with(component::Controlled::new(1))
            .build();

        // create planner and add systems

        let mut planner = specs::Planner::<WorldState>::new(world, 4);
        planner.add_system(system::Inertia::new(), "inertia", 15);
        planner.add_system(system::Render::new(), "render", 15);
        planner.add_system(system::Control::new(), "control", 15);

        // return level

        Level {
            planner: planner,
            inf: Arc::new(Infrastructure {
                scene   : scene,
                input   : input.clone(),
                sprite: laser,
                layer: effects,
            })
        }
    }

    pub fn process(self: &mut Self, renderer: &Renderer, delta: f32) {

        let world_state = WorldState {
            delta: delta,
            inf: self.inf.clone(),
        };

        self.planner.wait();
        renderer.draw_scene(&self.inf.scene, delta);
        self.planner.dispatch(world_state);
    }
}
