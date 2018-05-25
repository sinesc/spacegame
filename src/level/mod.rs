use prelude::*;
use specs;
use rodio;
use sound::{SoundGroup};
use def;
use def::FactionId;
use bloom;
use repository::Repository;
use specs::LazyUpdate;
use specs::world::EntitiesRes;

pub mod component;
mod system;

pub struct Infrastructure {
    input       : Input,
    audio       : rodio::Device,
    layer       : Repository<Arc<Layer>>,
    sprite      : Repository<Arc<Sprite>>,
    repository  : Repository<def::EntityDescriptor>,
    spawner     : Repository<def::SpawnerDescriptor, def::SpawnerId>,
    sound       : Repository<SoundGroup>,
    font        : Arc<Font>,
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
    pub fn spawn_lazy(self: &Self, lazy: &LazyUpdate, entities: &EntitiesRes, name: &str, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        self.inf.repository[name].spawn_lazy(lazy, entities, self.age, position, angle, faction);
    }
    pub fn spawner(self: &Self, lazy: &LazyUpdate, entities: &EntitiesRes, spawner_id: def::SpawnerId, parent_angle: Angle, parent_position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        let spawner = &self.inf.spawner.index(spawner_id);
        for ref spawn in &spawner.entities {
            if let Some(ref entity) = spawn.entity {
                let pos = match parent_position {
                    Some(parent_position) => parent_position + spawn.position.rotate(parent_angle),
                    None => spawn.position.rotate(parent_angle),
                };
                self.spawn_lazy(
                    lazy,
                    entities,
                    &entity,
                    Some(pos),
                    angle,
                    faction
                );
            }
            if let Some(ref sound) = spawn.sound {
                rodio::play_raw(&self.inf.audio, self.inf.sound[sound].samples());
            }
        }
    }
}

pub struct Level<'a, 'b> {
    world       : specs::World,
    dispatcher  : specs::Dispatcher<'a, 'b>,
    layer_def   : def::LayerDef,
    inf         : Arc<Infrastructure>,
    age         : f32,

    bloom       : postprocessors::Bloom,
    glare       : bloom::Bloom,
    roidspawn   : Periodic,
    minespawn   : Periodic,
    rng         : Rng,
    background  : Texture,
}

impl<'a, 'b> Level<'a, 'b> {

    pub fn new(input: &Input, context: &RenderContext) -> Self {

        // create world and register components

        let mut world = specs::World::new();
        world.register::<component::Bounding>();
        world.register::<component::Computed>();
        world.register::<component::Controlled>();
        world.register::<component::Explodes>();
        world.register::<component::Fading>();
        world.register::<component::Hitpoints>();
        world.register::<component::Inertial>();
        world.register::<component::Lifetime>();
        world.register::<component::Powerup>();
        world.register::<component::Shooter>();
        world.register::<component::Spatial>();
        world.register::<component::Visual>();

        // create a scene and a layer TODO: temporary, load from def

        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let background = Texture::from_file(context, "res/background/blue.jpg").unwrap();
        let audio = rodio::default_output_device().unwrap();

        // create layers

        let layer_def = def::parse_layers().unwrap();
        let mut layers = Repository::new();

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

        // load entity/spawner/faction definitions and required sprites

        let mut sprites = Repository::new();
        let factions = def::parse_factions().unwrap();
        let sounds = def::parse_sounds().unwrap();
        let spawners = def::parse_spawners().unwrap();
        let entities = def::parse_entities(&context, &mut sprites, &factions, &spawners, &layers).unwrap();

        // create player entity

        entities["player-1"].spawn(&mut world, 0., Some(Vec2(230., 350.)), None, None);

        let infrastructure = Arc::new(Infrastructure {
            audio       : audio,
            input       : input.clone(),
            layer       : layers,
            sprite      : sprites,
            repository  : entities,
            spawner     : spawners,
            sound       : sounds,
            font        : font,
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
                .with(system::Control, "control", &[])
                .with(system::Compute, "compute", &[])
                .with(system::Inertia, "inertia", &[ "control", "compute" ])
                .with(system::Collider, "collider", &[])
                .with(system::Upgrader, "upgrader", &[])
                .with(system::Render::new(), "render", &[ "control", "compute", "inertia", "collider", "upgrader" ])
                .with(system::Cleanup, "cleanup", &[ "render" ])
                .build();

        // return level

        let mut bloom = postprocessors::Bloom::new(&context, 4, 2);
        bloom.clear = false;
        bloom.draw_color = Color::alpha_pm(0.15);

        Level {
            world       : world,
            dispatcher  : dispatcher,
            layer_def   : layer_def,
            age         : 0.0,
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
        }

        self.age = age;
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

        // TODO: some temporary spawning

        if self.roidspawn.elapsed(age) {
            let angle = Angle(self.rng.range(-PI, PI));
            let mut pos = Vec2(800.0, 450.0) + Vec2::from(angle) * 2000.0;
            let outbound = pos.outbound(((0.0, 0.0), (1920.0, 1080.0))).unwrap();

            pos -= outbound;

            let v_max = Vec2::from(-angle) * 100.0;
            let faction = FactionId(self.rng.range(2., 100.) as usize);

            self.inf.repository["asteroid"].spawn(&mut self.world, self.age, Some(pos), Some(Angle::from(v_max)), Some(faction));
        }

        if self.minespawn.elapsed(age) {

            let angle = Angle(self.rng.range(-PI, PI));
            let mut pos = Vec2(800.0, 450.0) + Vec2::from(angle) * 2000.0;
            let outbound = pos.outbound(((0.0, 0.0), (1920.0, 1080.0))).unwrap();
            let faction = FactionId(self.rng.range(101., 200.) as usize);
            let pw_y = self.rng.range(0., 1080.);

            pos -= outbound;

            if self.rng.range(0., 1.) > 0.5 {
                self.inf.repository["mine-red"].spawn(&mut self.world, self.age, Some(pos), Some(angle), Some(faction));
            } else {
                self.inf.repository["mine-green"].spawn(&mut self.world, self.age, Some(pos), Some(angle), Some(faction));
            }

            if self.rng.range(0., 1.) > 0.5 {
                self.inf.repository["dual-weapon"].spawn(&mut self.world, self.age, Some(Vec2(1920., pw_y)), None, None);
            } else {
                self.inf.repository["triple-weapon"].spawn(&mut self.world, self.age, Some(Vec2(1920., pw_y)), None, None);
            }
        }

        self.world.maintain();
    }
}
