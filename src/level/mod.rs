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
    sprite      : HashMap<String, Arc<Sprite>>,
    repository  : HashMap<String, def::EntityDescriptor>,
    font        : Arc<Font>,
    pew         : SoundGroup,
    boom        : SoundGroup,
}

#[derive(Clone)]
pub struct WorldState {
    age         : f32,
    delta       : f32,
    take_input  : bool,
    paused      : bool,
    inf         : Arc<Infrastructure>,
}

impl WorldState {
    pub fn spawn_lazy(self: &Self, lazy: &specs::LazyUpdate, entities: &specs::world::EntitiesRes, name: &str, position: Option<Vec2>, angle: Option<Angle>, faction: Option<u32>) {
        self.inf.repository[name].spawn_lazy(lazy, entities, self.age, position, angle, faction);
    }
}

pub struct Level<'a, 'b> {
    world       : specs::World,
    dispatcher  : specs::Dispatcher<'a, 'b>,
    layer_def   : def::LayerDef,
    inf         : Arc<Infrastructure>,
    bloom       : postprocessors::Bloom,
    glare       : bloom::Bloom,

    roidspawn   : Periodic,
    minespawn   : Periodic,
    rng         : Rng,
    background  : Texture,
}

impl<'a, 'b> Level<'a, 'b> {

    pub fn spawn(self: &mut Self, name: &str, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<u32>) {
        self.inf.repository[name].spawn(&mut self.world, age, position, angle, faction);
    }

    pub fn new(input: &Input, context: &RenderContext) -> Self {

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

        // create a scene and a layer TODO: temporary, load from def

        let mut sprites = HashMap::new();
        sprites.insert("mine".to_string(), Sprite::from_file(context, "res/sprite/hostile/mine_red_lm_64x64x15x2.png").unwrap().arc());
        sprites.insert("friend".to_string(), Sprite::from_file(context, "res/sprite/player/speedy_98x72x30.png").unwrap().arc());
        sprites.insert("asteroid".to_string(), Sprite::from_file(context, "res/sprite/asteroid/type1_64x64x60.png").unwrap().arc());
        sprites.insert("explosion".to_string(), Sprite::from_file(context, "res/sprite/explosion/default_256x256x40.jpg").unwrap().arc());
        sprites.insert("laser".to_string(), Sprite::from_file(context, "res/sprite/projectile/bolt_white_60x36x1.jpg").unwrap().arc());

        sprites.insert("hostile/mine_green_lm_64x64x15x2.png".to_string(), Sprite::from_file(context, "res/sprite/hostile/mine_green_lm_64x64x15x2.png").unwrap().arc());
        sprites.insert("player/speedy_98x72x30.png".to_string(), Sprite::from_file(context, "res/sprite/player/speedy_98x72x30.png").unwrap().arc());
        sprites.insert("placeholder_16x16x1.png".to_string(), Sprite::from_file(context, "res/sprite/placeholder_16x16x1.png").unwrap().arc());
        sprites.insert("projectile/bolt_white_60x36x1.jpg".to_string(), Sprite::from_file(context, "res/sprite/projectile/bolt_white_60x36x1.jpg").unwrap().arc());
        sprites.insert("explosion/default_256x256x40.jpg".to_string(), Sprite::from_file(context, "res/sprite/explosion/default_256x256x40.jpg").unwrap().arc());
        sprites.insert("hostile/mine_red_lm_64x64x15x2.png".to_string(), Sprite::from_file(context, "res/sprite/hostile/mine_red_lm_64x64x15x2.png").unwrap().arc());
        sprites.insert("asteroid/type1_64x64x60.png".to_string(), Sprite::from_file(context, "res/sprite/asteroid/type1_64x64x60.png").unwrap().arc());


        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let background = Texture::from_file(context, "res/background/blue.jpg").unwrap();

        let audio = rodio::default_output_device().unwrap();
        let pew = SoundGroup::load(&["res/sound/projectile/pew1a.ogg", "res/sound/projectile/pew1b.ogg", "res/sound/projectile/pew1c.ogg", "res/sound/projectile/pew2.ogg"]).unwrap();
        let boom = SoundGroup::load(&["res/sound/damage/explosion_pop1.ogg", "res/sound/damage/explosion_pop2.ogg"]).unwrap();

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

        let factions = def::parse_factions().unwrap();
        let entities = def::parse_entities(&factions, &sprites, &layers).unwrap();

        //test
        entities["mine-green"].spawn(&mut world, 0., Some(Vec2(100., 100.)), None, None);

        // create player entity

        entities["player-1"].spawn(&mut world, 0., Some(Vec2(230., 350.)), None, None);

        let infrastructure = Arc::new(Infrastructure {
            input       : input.clone(),
            layer       : layers,
            sprite      : sprites,
            repository  : entities,
            font        : font,
            audio       : audio,
            pew         : pew,
            boom        : boom,
        });

        world.add_resource(WorldState {
            delta       : 0.0,
            age         : 0.0,
            take_input  : true,
            paused      : false,
            inf         : infrastructure.clone()
        });

        // create planner and add systems

        let dispatcher = specs::DispatcherBuilder::new()
                .with(system::Control::new(), "control", &[])
                .with(system::Compute::new(), "compute", &[])
                .with(system::Inertia::new(), "inertia", &[ "control", "compute" ])
                .with(system::Collider::new(), "collider", &[])
                .with(system::Render::new(), "render", &[ "control", "compute", "inertia", "collider" ])
                .with(system::Cleanup::new(), "cleanup", &[ "render" ])
                .build();

        // return level

        let mut bloom = postprocessors::Bloom::new(&context, 4, 2);
        bloom.clear = false;
        bloom.draw_color = Color::alpha_pm(0.15);

        Level {
            world       : world,
            dispatcher  : dispatcher,
            layer_def   : layer_def,
            roidspawn   : Periodic::new(0.0, 0.5),
            minespawn   : Periodic::new(0.0, 3.73),
            rng         : Rng::new(123.4),
            bloom       : bloom,
            glare       : bloom::Bloom::new(&context, (1920, 1080), 2, 5, 5.0),
            inf         : infrastructure,
            background  : background,
        }
    }

    pub fn process(self: &mut Self, renderer: &Renderer, age: f32, delta: f32, take_input: bool, paused: bool) {

        {
            let mut world_state = self.world.write_resource::<WorldState>();
            world_state.age = age;
            world_state.delta = delta;
            world_state.take_input = take_input;
            world_state.paused = paused;
        };

        self.dispatcher.dispatch(&mut self.world.res);

        // render layers

        renderer.fill().texture(&self.background).blendmode(blendmodes::COPY).draw();

        self.inf.font.write(&self.inf.layer["text"],
            "Mouse: move, R-Shift+Mouse: strafe, R-Ctrl+Mouse: rotate, Button1: shoot",
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

        // some temporary spawning

        if self.roidspawn.elapsed(age) {
            let angle = Angle(self.rng.range(-PI, PI));
            let mut pos = Vec2(800.0, 450.0) + angle.to_vec2() * 2000.0;
            let outbound = pos.outbound(((0.0, 0.0), (1920.0, 1080.0))).unwrap();
            let scale = self.rng.range(0.3, 1.3);

            pos -= outbound;

            let v_max = (-angle).to_vec2() * 100.0;
            let faction = self.rng.range(2., 100.) as u32;

            self.spawn("asteroid", age, Some(pos), Some(v_max.to_angle()), Some(faction));
            /*self.world.create_entity()
                .with(component::Spatial::new(pos, angle))
                .with(component::Visual::new(Some(self.inf.layer["base"].clone()), None, self.inf.sprite["asteroid"].clone(), Color::WHITE, scale, 30, 1.0))
                .with(component::Inertial::new(v_max, Vec2(1.0, 1.0), 1.0))
                .with(component::Bounding::new(20.0 * scale, self.rng.range(2., 100.) as u32))
                .with(component::Hitpoints::new(100. * scale))
                .build();*/

        }

        if self.minespawn.elapsed(age) {

            let angle = Angle(self.rng.range(-PI, PI));
            let mut pos = Vec2(800.0, 450.0) + angle.to_vec2() * 2000.0;
            let outbound = pos.outbound(((0.0, 0.0), (1920.0, 1080.0))).unwrap();
            let scale = self.rng.range(0.9, 1.1);
            let faction = self.rng.range(101., 200.) as u32;

            pos -= outbound;

            self.spawn("mine-red", age, Some(pos), Some(angle), Some(faction));
        }

        self.world.maintain();
    }
}
