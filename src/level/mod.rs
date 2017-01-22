use specs;
use radiant_rs::*;
use radiant_rs::scene::*;
use std::sync::Arc;
use std::time::Instant;

mod component;
mod system;

pub struct Infrastructure {
    scene   : Scene,
    input   : Input,

    sprite: SpriteId,
    layer: LayerId,
    font: FontId,
}

#[derive(Clone)]
pub struct WorldState {
    delta   : f32,
    age     : f32,
    inf     : Arc<Infrastructure>,
}

pub struct Level {
    planner : specs::Planner<WorldState>,
    inf     : Arc<Infrastructure>,
    created : Instant,
}

impl Level {

    pub fn new(input: &Input, context: &RenderContext) -> Level {

        // create world and register components

        let mut world = specs::World::new();
        world.register::<component::Spatial>();
        world.register::<component::Inertial>();
        world.register::<component::Visual>();
        world.register::<component::Controlled>();
        world.register::<component::Lifetime>();
        world.register::<component::Shooter>();

        // create a scene and a layer

        let font = Font::from_info(&context, FontInfo { family: "Arial".to_string(), size: 12.0, ..FontInfo::default() } );
        let scene = Scene::new(context);
        let base = scene.register_layer(1600, 900);
        let effects = scene.register_layer(1600, 900);
        let hostile = scene.register_sprite_from_file("res/sprite/hostile/mine_red_64x64x15.png");
        let friend = scene.register_sprite_from_file("res/sprite/player/speedy_98x72x30.png");
        let powerup = scene.register_sprite_from_file("res/sprite/powerup/ball_v_32x32x18.jpg");

        let laser = scene.register_sprite_from_file("res/sprite/projectile/bolt_white_60x36x1.jpg");
        let font = scene.register_font(font);

        scene.op(Op::SetBlendmode(base, blendmodes::ALPHA));
        scene.op(Op::SetBlendmode(effects, blendmodes::LIGHTEN));
        scene.op(Op::Draw(base));
        scene.op(Op::Draw(effects));
        scene.op(Op::Clear(base));
        scene.op(Op::Clear(effects));
        //scene.op(Op::RotateModelMatrixAt(base, 1.0, Vec2(0.0, 0.0), 0.1));

        // create test entity

        world.create_now()
            .with(component::Spatial::new(Vec2(230.0, 350.0), Angle(0.0)))
            .with(component::Visual::new(base, friend, 0))
            .with(component::Inertial::new(Vec2(800.0, 800.0), Vec2(0.0, 0.0), 4.0, 1.0))
            .with(component::Controlled::new(1))
            .with(component::Shooter::new(0.05))
            .build();

        world.create_now()
            .with(component::Spatial::new(Vec2(512.0, 384.0), Angle(0.0)))
            .with(component::Visual::new(base, friend, 0))
            .with(component::Inertial::new(Vec2(800.0, 800.0), Vec2(0.0, 0.0), 4.0, 1.0))
            .with(component::Controlled::new(2))
            .with(component::Shooter::new(0.15))
            .build();

        world.create_now()
            .with(component::Spatial::new(Vec2(120.0, 640.0), Angle(0.0)))
            .with(component::Visual::new(base, hostile, 30))
            .build();

        world.create_now()
            .with(component::Spatial::new(Vec2(530.0, 450.0), Angle(0.0)))
            .with(component::Visual::new(effects, powerup, 30))
            .build();

        // create planner and add systems

        let mut planner = specs::Planner::<WorldState>::new(world, 4);
        planner.add_system(system::Cleanup::new(), "cleanup", 100);
        planner.add_system(system::Control::new(), "control", 75);
        planner.add_system(system::Inertia::new(), "inertia", 50);
        planner.add_system(system::Render::new(), "render", 0);

        // return level

        Level {
            planner: planner,
            created: Instant::now(),
            inf: Arc::new(Infrastructure {
                scene   : scene,
                input   : input.clone(),
                sprite: laser,
                layer: effects,
                font: font,
            })
        }
    }

    pub fn process(self: &mut Self, renderer: &Renderer, delta: f32) {

        let age = Instant::now() - self.created;

        let world_state = WorldState {
            delta   : if delta.is_nan() || delta == 0.0 { 0.0001 } else { delta },
            age     : age.as_secs() as f32 + (age.subsec_nanos() as f64 / 1000000000.0) as f32,
            inf     : self.inf.clone(),
        };

        self.planner.wait();
        renderer.draw_scene(&self.inf.scene, delta);
        self.planner.dispatch(world_state);
    }
}
