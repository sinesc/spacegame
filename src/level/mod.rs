use crate::prelude::*;
use hecs;
use rodio;
use crate::sound::{SoundGroup};
use crate::def;
use crate::def::FactionId;
use crate::bloom;
use crate::repository::Repository;

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
    pub age         : f32,
    pub delta       : f32,
    pub take_input  : bool,
    pub paused      : bool,
    pub inf         : Arc<Infrastructure>,
}

impl WorldState {
    pub fn spawn_lazy(&self, cmd: &mut hecs::CommandBuffer, name: &str, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        self.inf.repository[name].spawn_lazy(cmd, self.age, position, angle, faction);
    }
    pub fn spawner(&self, cmd: &mut hecs::CommandBuffer, spawner_id: def::SpawnerId, parent_angle: Angle, parent_position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        let spawner = &self.inf.spawner.index(spawner_id);
        for spawn in &spawner.entities {
            let pos = match parent_position {
                Some(parent_position) => parent_position + spawn.position.rotate(parent_angle),
                None => spawn.position.rotate(parent_angle),
            };
            if let Some(ref entity) = spawn.extend {
                entity.get().unwrap().spawn_lazy(cmd, self.age, Some(pos), angle, faction);
            }
            if let Some(ref sound) = spawn.sound {
                rodio::play_raw(&self.inf.audio, self.inf.sound[sound].samples());
            }
        }
    }
}

pub struct Level {
    world           : hecs::World,
    world_state     : WorldState,
    render_system   : system::Render,
    layer_def       : def::LayerDef,
    inf             : Arc<Infrastructure>,
    age             : f32,

    bloom           : postprocessors::Bloom,
    glare           : bloom::Bloom,
    roidspawn       : Periodic,
    minespawn       : Periodic,
    rng             : Rng,
    background      : Texture,
}

impl Level {

    pub fn new(input: &Input, context: &Context) -> Self {

        let mut world = hecs::World::new();

        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let background = Texture::from_file(context, "res/background/blue.jpg").unwrap();
        let audio = rodio::default_output_device().unwrap();

        let layer_def = def::parse_layers().unwrap();
        let mut layers = Repository::new();

        for info in &layer_def.create {
            let layer = Layer::new((info.scale * 1920., info.scale * 1080.)).arc();
            if let Some(ref blendmode) = info.blendmode {
                if blendmode == "add" {
                    layer.set_blendmode(blendmodes::ADD);
                } else if blendmode == "lighten" {
                    layer.set_blendmode(blendmodes::LIGHTEN);
                }
            }
            layers.insert(info.name.clone(), layer);
        }

        let mut sprites = Repository::new();
        let factions = def::parse_factions().unwrap();
        let sounds = def::parse_sounds().unwrap();
        let mut spawners = def::parse_spawners().unwrap();
        let entities = def::parse_entities(&context, &mut sprites, &factions, &mut spawners, &layers).unwrap();

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

        let world_state = WorldState {
            delta       : 0.0,
            age         : 0.0,
            take_input  : true,
            paused      : false,
            inf         : infrastructure.clone(),
        };

        let mut bloom = postprocessors::Bloom::new(&context, (1920u32, 1080u32), 2);
        bloom.clear = false;
        bloom.draw_color = Color::alpha_pm(0.15);

        Level {
            world           : world,
            world_state     : world_state,
            render_system   : system::Render::new(),
            layer_def       : layer_def,
            age             : 0.0,
            roidspawn       : Periodic::new(0.0, 0.5),
            minespawn       : Periodic::new(0.0, 3.73),
            rng             : Rng::new(123.4),
            bloom           : bloom,
            glare           : bloom::Bloom::new(&context, (1920, 1080), 2, 5, 5.0),
            inf             : infrastructure,
            background      : background,
        }
    }

    pub fn process(&mut self, renderer: &Renderer, age: f32, delta: f32, take_input: bool, paused: bool) {

        self.world_state.age = age;
        self.world_state.delta = delta;
        self.world_state.take_input = take_input;
        self.world_state.paused = paused;
        self.age = age;

        let mut cmd = hecs::CommandBuffer::new();

        system::run_control(&mut self.world, &self.world_state, &mut cmd);
        system::run_compute(&mut self.world, &self.world_state, &mut cmd);
        system::run_inertia(&mut self.world, &self.world_state);
        system::run_collider(&mut self.world, &self.world_state, &mut cmd);
        system::run_upgrader(&mut self.world, &self.world_state, &mut cmd);
        self.render_system.run(&mut self.world, &self.world_state);
        system::run_cleanup(&mut self.world, &self.world_state);

        cmd.run_on(&mut self.world);

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
    }
}
