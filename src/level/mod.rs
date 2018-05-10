use prelude::*;
use specs;
use rodio;
use sound::{SoundGroup};
use def;
use bloom;

pub mod component;
mod system;

pub struct Infrastructure {
    input       : Input,
    audio       : rodio::Device,
    layer       : HashMap<String, Arc<Layer>>,
    font        : Arc<Font>,
    sprite      : Arc<Sprite>,
    asteroid    : Arc<Sprite>,
    explosion   : Arc<Sprite>,
    mine        : Arc<Sprite>,
    pew         : SoundGroup,
    boom        : SoundGroup,
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
    layer_def   : def::LayerDef,
    created     : Instant,


    inf         : Arc<Infrastructure>,
    roidspawn   : Periodic,
    minespawn   : Periodic,
    rng         : Rng,
    bloom       : postprocessors::Bloom,
    glare       : bloom::Bloom,
    background  : Texture,
}

impl<'a, 'b> Level<'a, 'b> {

    pub fn new(input: &Input, context: &RenderContext) -> Level<'a, 'b> {

        // create world and register components

        let mut world = specs::World::new();
        world.register::<component::Spatial>();
        world.register::<component::Inertial>();
        world.register::<component::Visual>();
        world.register::<component::Controlled>();
        world.register::<component::Computed>();
        world.register::<component::Lifetime>();
        world.register::<component::Shooter>();
        world.register::<component::Fading>();
        world.register::<component::Bounding>();
        world.register::<component::Hitpoints>();

        // create a scene and a layer

        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let mine = Sprite::from_file(context, "res/sprite/hostile/mine_lightmapped_64x64x15x2.png").unwrap().arc();
        //let mine = Sprite::from_file(context, "res/sprite/hostile/mine_red_64x64x15.png").unwrap().arc();
        let friend = Sprite::from_file(context, "res/sprite/player/speedy_98x72x30.png").unwrap().arc();
        let asteroid = Sprite::from_file(context, "res/sprite/asteroid/type1_64x64x60.png").unwrap().arc();
        let explosion = Sprite::from_file(context, "res/sprite/explosion/default_256x256x40.jpg").unwrap().arc();
        //let powerup = Sprite::from_file(context, "res/sprite/powerup/ball_v_32x32x18.jpg").unwrap().arc();
        // res/sprite/explosion/lightmapped_256x256x40x2.jpg
        // res/sprite/explosion/default_256x256x40.jpg

        let laser = Sprite::from_file(context, "res/sprite/projectile/bolt_white_60x36x1.jpg").unwrap().arc();
        let background = Texture::from_file(context, "res/background/blue.jpg").unwrap();

        let audio = rodio::default_output_device().unwrap();
        let pew = SoundGroup::load(&["res/sound/projectile/pew1a.ogg", "res/sound/projectile/pew1b.ogg", "res/sound/projectile/pew1c.ogg", "res/sound/projectile/pew2.ogg"]).unwrap();
        let boom = SoundGroup::load(&["res/sound/damage/explosion_pop1.ogg", "res/sound/damage/explosion_pop2.ogg"]).unwrap();

        let tmp = def::parse_entities().unwrap();
println!("{:?}", tmp);
        // create layers

        let layer_def = def::parse_layers().unwrap();
        let mut layers = HashMap::new();

        for info in &layer_def.create {
            let mut layer = Layer::new((info.scale * 1920., info.scale * 1080.)).arc();
            // todo: meh, have serde map the json string to the blendmode somehow (enum?)
            if let Some(ref blendmode) = info.blendmode {
                if blendmode == "add" {
                    layer.set_blendmode(blendmodes::ADD);
                } else if blendmode == "lighten" {
                    layer.set_blendmode(blendmodes::LIGHTEN);
                }
            }
            layers.insert(info.name.clone(), layer);
        }

        // create player entity

        world.create_entity()
            .with(component::Spatial::new(Vec2(230.0, 350.0), Angle(0.0)))
            .with(component::Visual::new(Some(layers["base"].clone()), None, friend.clone(), Color(0.8, 0.8, 1.0, 1.0), 1.0, 0, 1.0))
            .with(component::Inertial::new(Vec2(1200.0, 1200.0), Vec2(0.0, 0.0), 7.0))
            .with(component::Controlled::new(1))
            .with(component::Shooter::new(0.2))
            .with(component::Bounding::new(20.0, 1))
            .with(component::Hitpoints::new(100000.))
            .build();
/*
        world.create_entity()
            .with(component::Spatial::new(Vec2(512.0, 384.0), Angle(0.0), true))
            .with(component::Visual::new(Some(base.clone()), None, friend.clone(), Color(1.0, 0.8, 0.8, 1.0), 0, 1.0))
            .with(component::Inertial::new(Vec2(1200.0, 1200.0), Vec2(0.0, 0.0), 1.0))
            .with(component::Controlled::new(2))
            .with(component::Shooter::new(0.02))
            .with(component::Bounding::new(20.0, 1))
            .with(component::Hitpoints::new(100.))
            .build();

        world.create_entity()
            .with(component::Spatial::new(Vec2(530.0, 450.0), Angle(0.0), true))
            .with(component::Visual::new(Some(effects.clone()), None, powerup.clone(), Color::WHITE, 30, 1.0))
            .with(component::Bounding::new(20.0, 0))
            .with(component::Hitpoints::new(100.))
            .build();
*/

        let infrastructure = Arc::new(Infrastructure {
            input       : input.clone(),
            layer       : layers,
            sprite      : laser,
            asteroid    : asteroid,
            mine        : mine,
            font        : font,
            explosion   : explosion,
            audio       : audio,
            pew         : pew,
            boom        : boom,
        });

        world.add_resource(WorldState { delta: 0.0, age: 0.0, inf: infrastructure.clone() });

        // create planner and add systems

        let dispatcher = specs::DispatcherBuilder::new()
                .add(system::Control::new(), "control", &[])
                .add(system::Compute::new(), "compute", &[])
                .add(system::Inertia::new(), "inertia", &[ "control", "compute" ])
                .add(system::Collider::new(), "collider", &[])
                .add(system::Render::new(), "render", &[ "control", "compute", "inertia", "collider" ])
                .add(system::Cleanup::new(), "cleanup", &[ "render" ])
                .build();

        // return level

        let created = Instant::now();

        let mut bloom = postprocessors::Bloom::new(&context, 4, 2);
        bloom.clear = false;
        bloom.draw_color = Color::alpha_pm(0.15);

        Level {
            world       : world,
            dispatcher  : dispatcher,
            layer_def   : layer_def,
            created     : created,
            roidspawn   : Periodic::new(0.0, 0.5),
            minespawn   : Periodic::new(0.0, 3.73),
            rng         : Rng::new(123.4),
            bloom       : bloom,
            glare       : bloom::Bloom::new(&context, (1920, 1080), 2, 5, 5.0),
            inf         : infrastructure,
            background  : background,
        }
    }

    pub fn process(self: &mut Self, renderer: &Renderer, delta: f32) {

        let age = Instant::now() - self.created;
        let age = age.as_secs() as f32 + (age.subsec_nanos() as f64 / 1000000000.0) as f32;

        {
            let mut world_state = self.world.write_resource::<WorldState>();
            world_state.delta = if delta.is_nan() || delta == 0.0 { 0.0001 } else { delta };
            world_state.age = age;
        }

        self.dispatcher.dispatch(&mut self.world.res);

        renderer.fill().texture(&self.background).blendmode(blendmodes::COPY).draw();

        self.inf.font.write(&self.inf.layer["text"],
            &("Mouse: move, Shift+Mouse: strafe, Button1: shoot"),
            (10.0, 740.0),
            Color::WHITE
        );

        for info in &self.layer_def.render {
            if let Some(ref filter) = info.filter {
                if filter == "bloom" {
                    renderer.postprocess(&self.bloom, &(), || {
                        renderer.fill().color(Color::alpha_mask(0.3)).draw();
                        renderer.draw_layer(&self.inf.layer[&info.name], info.component);
                    });
                } else if filter == "glare" {
                    renderer.postprocess(&self.glare, &blendmodes::SCREEN, || {
                        renderer.fill().color(Color::alpha_mask(0.05)).draw();
                        renderer.draw_layer(&self.inf.layer[&info.name], info.component);
                    });
                } else {
                    panic!("invalid filter name");
                }
            } else {
                renderer.draw_layer(&self.inf.layer[&info.name], 0);
            }
        }

        for info in &self.layer_def.create {
            self.inf.layer[&info.name].clear();
        }

        if self.roidspawn.elapsed(age) {
            let angle = Angle(self.rng.range(-PI, PI));
            let mut pos = Vec2(800.0, 450.0) + angle.to_vec2() * 2000.0;
            let outbound = pos.outbound(((0.0, 0.0), (1920.0, 1080.0))).unwrap();
            let scale = self.rng.range(0.3, 1.3);

            pos -= outbound;

            let v_max = (-angle).to_vec2() * 100.0;

            self.world.create_entity()
                .with(component::Spatial::new(pos, angle))
                .with(component::Visual::new(Some(self.inf.layer["base"].clone()), None, self.inf.asteroid.clone(), Color::WHITE, scale, 30, 1.0))
                .with(component::Inertial::new(v_max, Vec2(1.0, 1.0), 1.0))
                .with(component::Bounding::new(20.0 * scale, self.rng.range(2., 100.) as u32))
                .with(component::Hitpoints::new(100. * scale))
                .build();

        }

        if self.minespawn.elapsed(age) {
            let angle = Angle(self.rng.range(-PI, PI));
            let mut pos = Vec2(800.0, 450.0) + angle.to_vec2() * 2000.0;
            let outbound = pos.outbound(((0.0, 0.0), (1920.0, 1080.0))).unwrap();
            let scale = self.rng.range(0.9, 1.1);

            pos -= outbound;

            self.world.create_entity()
                .with(component::Spatial::new(pos, angle))
                .with(component::Visual::new(Some(self.inf.layer["base"].clone()), None, self.inf.mine.clone(), Color::WHITE, scale, 30, 1.0))
                .with(component::Bounding::new(20.0, self.rng.range(101., 200.) as u32))
                .with(component::Hitpoints::new(100.))
                .with(component::Shooter::new(0.5))
                .with(component::Computed::new())
                .with(component::Inertial::new(Vec2(120.0, 120.0), Vec2(0.0, 0.0), 1.0))
                .build();
        }

        self.world.maintain();
    }
}
