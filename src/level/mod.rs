use specs;
use radiant_rs::*;
use radiant_rs::math::*;
use std::sync::Arc;
use std::time::Instant;
use std::f32::consts::PI;

mod component;
mod system;

pub struct Infrastructure {
    input       : Input,
    effects     : Arc<Layer>,
    base        : Arc<Layer>,
    font        : Arc<Font>,
    sprite      : Arc<Sprite>,
    asteroid    : Arc<Sprite>,
    explosion   : Arc<Sprite>,
}

#[derive(Clone)]
pub struct WorldState {
    delta   : f32,
    age     : f32,
    inf     : Arc<Infrastructure>,
}

pub struct Level<'a, 'b> {
    world       : specs::World,
    dispatcher  : specs::Dispatcher<'a, 'b>,
    inf         : Arc<Infrastructure>,
    roidspawn   : utils::Periodic,
    rng         : utils::Rng,
    bloom       : Arc<super::post::Bloom>,
    created     : Instant,
}

impl<'a, 'b> Level<'a, 'b> {

    pub fn new(input: &Input, context: &RenderContext) -> Level<'a, 'b> {

        // create world and register components

        let mut world = specs::World::new();
        world.register::<component::Spatial>();
        world.register::<component::Inertial>();
        world.register::<component::Visual>();
        world.register::<component::Controlled>();
        world.register::<component::Lifetime>();
        world.register::<component::Shooter>();
        world.register::<component::Fading>();
        world.register::<component::Bounding>();
        world.register::<component::Hitpoints>();

        // create a scene and a layer

        let base = Layer::new((1600., 900.)).arc();
        let effects = Layer::new((1600., 900.)).arc();
        //let bloom = Layer::new((1600., 900.)).arc();

        effects.set_blendmode(blendmodes::LIGHTEN);
        //bloom.set_blendmode(blendmodes::LIGHTEN);

        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let hostile = Sprite::from_file(context, "res/sprite/hostile/mine_red_64x64x15.png").unwrap().arc();
        let friend = Sprite::from_file(context, "res/sprite/player/speedy_98x72x30.png").unwrap().arc();
        let powerup = Sprite::from_file(context, "res/sprite/powerup/ball_v_32x32x18.jpg").unwrap().arc();
        let asteroid = Sprite::from_file(context, "res/sprite/asteroid/type1_64x64x60.png").unwrap().arc();
        let explosion = Sprite::from_file(context, "res/sprite/explosion/default_256x256x40.jpg").unwrap().arc();
        let laser = Sprite::from_file(context, "res/sprite/projectile/bolt_white_60x36x1.jpg").unwrap().arc();

        // create test entity

        world.create_entity()
            .with(component::Spatial::new(Vec2(230.0, 350.0), Angle(0.0), true))
            .with(component::Visual::new(Some(base.clone()), None, friend.clone(), Color(0.8, 0.8, 1.0, 1.0), 0, 1.0))
            .with(component::Inertial::new(Vec2(1200.0, 1200.0), Vec2(0.0, 0.0), 4.0, 1.5))
            .with(component::Controlled::new(1))
            .with(component::Shooter::new(0.05))
            .with(component::Bounding::new(20.0, 1))
            .with(component::Hitpoints::new(100.))
            .build();

        world.create_entity()
            .with(component::Spatial::new(Vec2(512.0, 384.0), Angle(0.0), true))
            .with(component::Visual::new(Some(base.clone()), None, friend.clone(), Color(1.0, 0.8, 0.8, 1.0), 0, 1.0))
            .with(component::Inertial::new(Vec2(1200.0, 1200.0), Vec2(0.0, 0.0), 4.0, 1.5))
            .with(component::Controlled::new(2))
            .with(component::Shooter::new(0.05))
            .with(component::Bounding::new(20.0, 1))
            .with(component::Hitpoints::new(100.))
            .build();

        world.create_entity()
            .with(component::Spatial::new(Vec2(120.0, 640.0), Angle(0.0), true))
            .with(component::Visual::new(Some(base.clone()), None, hostile.clone(), Color::WHITE, 30, 1.0))
            .with(component::Bounding::new(20.0, 0))
            .with(component::Hitpoints::new(100.))
            .build();

        world.create_entity()
            .with(component::Spatial::new(Vec2(530.0, 450.0), Angle(0.0), true))
            .with(component::Visual::new(Some(effects.clone()), None, powerup.clone(), Color::WHITE, 30, 1.0))
            .with(component::Bounding::new(20.0, 0))
            .with(component::Hitpoints::new(100.))
            .build();


        let infrastructure = Arc::new(Infrastructure {
            input       : input.clone(),
            base        : base,
            effects     : effects,
            sprite      : laser,
            asteroid    : asteroid,
            font        : font,
            explosion   : explosion,
        });

        world.add_resource(WorldState { delta: 0.0, age: 0.0, inf: infrastructure.clone() });

        // create planner and add systems

        let dispatcher = specs::DispatcherBuilder::new()
                .add(system::Control::new(), "control", &[])
                .add(system::Inertia::new(), "inertia", &[ "control" ])
                .add(system::Collider::new(), "collider", &[])
                .add(system::Render::new(), "render", &[ "control", "inertia", "collider" ])
                .add(system::Cleanup::new(), "cleanup", &[ "render" ])
                .build();

        // return level

        let created = Instant::now();

        Level {
            world       : world,
            dispatcher  : dispatcher,
            created     : created,
            roidspawn   : utils::Periodic::new(0.0, 0.2),
            rng         : utils::Rng::new(123.4),
            bloom       : Arc::new(super::post::Bloom::new(&context)),
            inf         : infrastructure
        }
    }

    pub fn process(self: &mut Self, renderer: &Renderer, delta: f32) {
        use std::ops::Deref;

        let age = Instant::now() - self.created;
        let age = age.as_secs() as f32 + (age.subsec_nanos() as f64 / 1000000000.0) as f32;

        {
            let mut world_state = self.world.write_resource::<WorldState>();

            *world_state = WorldState {
                delta   : if delta.is_nan() || delta == 0.0 { 0.0001 } else { delta },
                age     : age,
                inf     : self.inf.clone(),
            };
        }

        self.dispatcher.dispatch(&mut self.world.res);

        let bloom_args = super::post::BloomArgs {
            iterations  : 4,
            iter_blend  : blendmodes::COPY,
            final_blend : blendmodes::LIGHTEN,
            spread      : 5,
            color       : Color::alpha_pm(0.25),
        };

        renderer.clear(Color(0.0, 0.0, 0.0, 1.0));

        renderer.postprocess(self.bloom.deref(), &bloom_args, || {
            renderer.clear(Color(0.0, 0.0, 0.0, 1.0));
            renderer.draw_layer(&self.inf.effects, 0);
        });

        self.inf.font.write(&self.inf.base,
            &("Player1: Cursor: move, Ctrl-Right: fire, Shift-Right + Up/Down: rotate, Shift-Right + Left/Right: forward/backward\r\n".to_string() +
            "Player2: WASD: move, Ctrl-Left: fire, Shift-Left + WS: rotate, Shift-Left + AD: forward/backward"),
            Vec2(10.0, 740.0),
            Color::WHITE
        );

        renderer.draw_layer(&self.inf.base, 0);
        renderer.draw_layer(&self.inf.effects, 0);

        self.inf.base.clear();
        self.inf.effects.clear();

        if self.roidspawn.elapsed(age) {
            let angle = Angle(self.rng.range(-PI, PI));
            let mut pos = Vec2(800.0, 450.0) + angle.to_vec2() * 2000.0;
            let outbound = pos.outbound(Rect::new(0.0, 0.0, 1600.0, 900.0)).unwrap();

            pos -= outbound;

            let v_max = (-angle).to_vec2() * 100.0;

            self.world.create_entity()
                .with(component::Spatial::new(pos, angle, true))
                .with(component::Visual::new(Some(self.inf.base.clone()), None, self.inf.asteroid.clone(), Color::WHITE, 30, 1.0))
                .with(component::Inertial::new(v_max, Vec2(1.0, 1.0), 4.0, 1.5))
                .with(component::Bounding::new(20.0, 2))
                .with(component::Hitpoints::new(100.))
                .build();

        }

        self.world.maintain();
    }
}
