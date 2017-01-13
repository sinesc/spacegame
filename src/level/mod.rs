use specs;
use radiant_rs::*;
use radiant_rs::scene::*;
use std::sync::Arc;

mod component;
mod system;

#[derive(Clone)]
pub struct WorldState {
    delta: f32,
}

pub struct Level {
    planner: specs::Planner<WorldState>,
    scene: Arc<Scene>,

}

impl Level {

    pub fn new(input: &Input, context: &RenderContext) -> Self {

        // create world and register components

        let mut world = specs::World::new();
        world.register::<component::Spatial>();
        world.register::<component::Inertial>();
        world.register::<component::Visual>();

        // create a scene and a layer

        let scene = Arc::new(Scene::new(context));
        let base = scene.register_layer(1024, 768);
        let effects = scene.register_layer(1024, 768);
        let hostile = scene.register_sprite_from_file("res/sprite/hostile/mine_red_64x64x15.png");
        let friend = scene.register_sprite_from_file("res/sprite/player/speedy_98x72x30.png");
        let powerup = scene.register_sprite_from_file("res/sprite/powerup/ball_v_32x32x18.jpg");

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
            .build();

        // create planner and add systems

        let mut planner = specs::Planner::<WorldState>::new(world, 4);
        planner.add_system(system::Inertia { }, "inertia", 15);
        planner.add_system(system::Render::new(&scene), "render", 15);

        // return level

        Level {
            scene   : scene.clone(),
            planner : planner
        }
    }

    pub fn process(self: &mut Self, renderer: &Renderer, delta: f32) {

        let world_state = WorldState {
            delta: delta,
        };

        self.planner.wait();
        renderer.draw_scene(&self.scene, delta);
        self.planner.dispatch(world_state);
    }
}
