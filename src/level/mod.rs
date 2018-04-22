use prelude::*;
use specs;
use rodio;

pub mod component;
mod system;

use std::io;
use std::convert::AsRef;

pub struct Sound (Arc<Vec<u8>>);

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Sound {
    pub fn load(filename: &str) -> io::Result<Sound> {
        use std::fs::File;
        //use std::io::BufReader;
        let mut buf = Vec::new();
        let mut file = File::open(filename)?;
        file.read_to_end(&mut buf)?;
        Ok(Sound(Arc::new(buf)))
    }
    pub fn cursor(self: &Self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }
    pub fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<Sound>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }
}


pub struct Infrastructure {
    input       : Input,
    audio       : rodio::Device,
    layer       : HashMap<String, Arc<Layer>>,
    font        : Arc<Font>,
    sprite      : Arc<Sprite>,
    asteroid    : Arc<Sprite>,
    explosion   : Arc<Sprite>,
    pew         : Sound,
    boom        : Sound,
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
    rng         : Rng,
    bloom       : postprocessors::Bloom,
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
        world.register::<component::Lifetime>();
        world.register::<component::Shooter>();
        world.register::<component::Fading>();
        world.register::<component::Bounding>();
        world.register::<component::Hitpoints>();

        // create a scene and a layer

        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        //let hostile = Sprite::from_file(context, "res/sprite/hostile/mine_red_64x64x15.png").unwrap().arc();
        let friend = Sprite::from_file(context, "res/sprite/player/speedy_98x72x30.png").unwrap().arc();
        //let powerup = Sprite::from_file(context, "res/sprite/powerup/ball_v_32x32x18.jpg").unwrap().arc();
        let asteroid = Sprite::from_file(context, "res/sprite/asteroid/type1_64x64x60.png").unwrap().arc();
        let explosion = Sprite::from_file(context, "res/sprite/explosion/default_256x256x40.jpg").unwrap().arc();
        // res/sprite/explosion/lightmapped_256x256x40x2.jpg
        // res/sprite/explosion/default_256x256x40.jpg

        let laser = Sprite::from_file(context, "res/sprite/projectile/bolt_white_60x36x1.jpg").unwrap().arc();
        let background = Texture::from_file(context, "res/background/blue.jpg").unwrap();

        let audio = rodio::default_output_device().unwrap();
        let pew = Sound::load("res/sound/projectile/pew1a.ogg").unwrap();
        let boom = Sound::load("res/sound/damage/explosion1.ogg").unwrap();

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

        // create test entity

        world.create_entity()
            .with(component::Spatial::new(Vec2(230.0, 350.0), Angle(0.0)))
            .with(component::Visual::new(Some(layers["base"].clone()), None, friend.clone(), Color(0.8, 0.8, 1.0, 1.0), 1.0, 0, 1.0))
            .with(component::Inertial::new(Vec2(1200.0, 1200.0), Vec2(0.0, 0.0), 4.0, 1.5, true))
            .with(component::Controlled::new(1))
            .with(component::Shooter::new(0.2))
            .with(component::Bounding::new(20.0, 1))
            .with(component::Hitpoints::new(100.))
            .build();
/*
        world.create_entity()
            .with(component::Spatial::new(Vec2(512.0, 384.0), Angle(0.0), true))
            .with(component::Visual::new(Some(base.clone()), None, friend.clone(), Color(1.0, 0.8, 0.8, 1.0), 0, 1.0))
            .with(component::Inertial::new(Vec2(1200.0, 1200.0), Vec2(0.0, 0.0), 4.0, 1.5))
            .with(component::Controlled::new(2))
            .with(component::Shooter::new(0.02))
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
*/

        let infrastructure = Arc::new(Infrastructure {
            input       : input.clone(),
            layer       : layers,
            sprite      : laser,
            asteroid    : asteroid,
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
            layer_def   : layer_def,
            created     : created,
            roidspawn   : Periodic::new(0.0, 0.1),
            rng         : Rng::new(123.4),
            bloom       : postprocessors::Bloom::new(&context, 4, 2),
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

        //renderer.clear(Color(0.0, 0.0, 0.0, 1.0));
        renderer.fill().texture(&self.background).blendmode(blendmodes::COPY).draw();

        self.inf.font.write(&self.inf.layer["base"],
            &("Mouse: move, Shift+Mouse: strafe, Button1: shoot"),
            (10.0, 740.0),
            Color::WHITE
        );

        self.bloom.draw_color = Color::alpha_pm(0.15);
        self.bloom.clear = false;

        for info in &self.layer_def.render {
            if let Some(ref filter) = info.filter {
                if filter == "bloom" {
                    renderer.postprocess(&self.bloom, &(), || {
                        renderer.fill().color(Color::alpha_mask(0.3)).draw();
                        renderer.draw_layer(&self.inf.layer[&info.name], info.component);
                    });
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
                .with(component::Inertial::new(v_max, Vec2(1.0, 1.0), 4.0, 1.5, true))
                .with(component::Bounding::new(20.0 * scale, self.rng.range(2., 102.) as u32))
                .with(component::Hitpoints::new(100. * scale))
                .build();

        }

        self.world.maintain();
    }
}
