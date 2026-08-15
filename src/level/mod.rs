use crate::prelude::*;
use hecs;
use rodio::{MixerDeviceSink, DeviceSinkBuilder};
use rodio::mixer::Mixer;
use crate::bloom;
use crate::scripting;
use crate::scripting::ScriptingSubsystem;

pub mod component;
mod system;

/// A filtered render pass on a layer (created by the Itsy script via
/// `add_render_layer`), mirroring the old layer.yaml "render" section.
#[derive(Clone, Copy, Debug)]
pub struct RenderLayer {
    pub layer_id  : u32,
    pub filter    : Option<RenderFilter>,
    pub component : u32,
}

#[derive(Clone, Copy, Debug)]
pub enum RenderFilter {
    Bloom,
    Glare,
}

pub struct Infrastructure {
    input         : Input,
    pub audio     : Mixer,
    /// Render layers created by the Itsy script (`create_layer`); the vector
    /// index is the layer ID shared between Itsy and Rust.
    pub layers    : Vec<Arc<Layer>>,
    /// Render passes created by the Itsy script (`add_render_layer`), in draw order.
    pub render_layers : Vec<RenderLayer>,
    pub font      : Arc<Font>,
    /// Layer ID used for Rust-side debug text (set by Itsy via `set_debug_layer`);
    /// `u32::MAX` = not set yet.
    pub debug_layer : std::sync::atomic::AtomicU32,
}

impl Infrastructure {
    /// The layer designated for Rust-side debug text, if the script set one.
    pub fn debug_layer(&self) -> Option<&Arc<Layer>> {
        let idx = self.debug_layer.load(std::sync::atomic::Ordering::Relaxed) as usize;
        self.layers.get(idx)
    }
}

#[derive(Clone)]
pub struct WorldState {
    pub age         : f32,
    pub delta       : f32,
    pub take_input  : bool,
    pub paused      : bool,
    pub inf         : Arc<Infrastructure>,
}

pub struct Level {
    world           : hecs::World,
    world_state     : WorldState,
    render_system   : system::Render,
    inf             : Arc<Infrastructure>,
    age             : f32,
    _audio_sink     : MixerDeviceSink,

    bloom           : postprocessors::Bloom,
    glare           : bloom::Bloom,
    background      : Texture,
    scripting       : ScriptingSubsystem,
    game_started    : bool,  // track if GAME_START trigger was sent
}

impl Level {

    pub fn new(input: &Input, context: &Context) -> Self {

        let world = hecs::World::new();

        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let background = Texture::from_file(context, "res/background/blue.jpg").unwrap();
        let mut audio_sink = DeviceSinkBuilder::open_default_sink().unwrap();
        audio_sink.log_on_drop(false);
        let audio = audio_sink.mixer().clone();

        // Render layers are created by the Itsy script (create_layer /
        // add_render_layer); player is spawned via the GAME_START trigger.

        let infrastructure = Arc::new(Infrastructure {
            audio         : audio,
            input         : input.clone(),
            layers        : Vec::new(),
            render_layers : Vec::new(),
            font          : font,
            debug_layer   : std::sync::atomic::AtomicU32::new(u32::MAX),
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

        // The scripting subsystem holds a raw pointer into the Infrastructure;
        // keep an Arc so it stays alive as long as the Level does.
        let inf_for_scripting = infrastructure.clone();

        Level {
            world           : world,
            world_state     : world_state,
            render_system   : system::Render::new(),
            age             : 0.0,
            _audio_sink     : audio_sink,
            bloom           : bloom,
            glare           : bloom::Bloom::new(&context, (1920, 1080), 2, 5, 5.0),
            inf             : infrastructure,
            background      : background,
            scripting       : ScriptingSubsystem::new(context.clone(), inf_for_scripting),
            game_started    : false,
        }
    }

    /// Detect collision pairs for the scripting subsystem.
    /// Returns flat list: [a, b, c, d, ...] = [(a,b), (c,d)].
    fn detect_collision_pairs(world: &hecs::World) -> Vec<u64> {
        let entities: Vec<(hecs::Entity, Vec2, f32, u16)> = world
            .query::<(&component::Spatial, &component::Bounding, &component::Hitpoints)>()
            .iter()
            .map(|(e, (s, b, _))| (e, s.position, b.radius, b.faction))
            .collect();

        let mut pairs = Vec::new();
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let &(ea, pos_a, rad_a, fac_a) = &entities[i];
                let &(eb, pos_b, rad_b, fac_b) = &entities[j];
                if fac_a != fac_b && rad_a + rad_b > pos_a.distance(&pos_b) {
                    pairs.push(ea.to_bits().into());
                    pairs.push(eb.to_bits().into());
                }
            }
        }
        pairs
    }

    pub fn process(&mut self, renderer: &Renderer, age: f32, delta: f32, take_input: bool, paused: bool) {

        self.world_state.age = age;
        self.world_state.delta = delta;
        self.world_state.take_input = take_input;
        self.world_state.paused = paused;
        self.age = age;

        let mut cmd = hecs::CommandBuffer::new();

        // Detect collisions for scripting subsystem
        let collision_pairs = Self::detect_collision_pairs(&self.world);
        self.scripting.set_collisions(collision_pairs);

        // Configure scripting subsystem
        self.scripting.set_game_time(age);
        // Firing input: mouse button 1 or left ctrl
        let firing = self.world_state.inf.input.down(InputId::Mouse1)
            || self.world_state.inf.input.down(InputId::LControl);
        self.scripting.set_input_fire(firing);

        // Pass mouse position and keyboard input
        let mouse_pos = self.world_state.inf.input.mouse();
        self.scripting.set_mouse_pos(mouse_pos.0 as f32, mouse_pos.1 as f32);

        // Pass mouse delta (reliable even when cursor is grabbed/focused)
        let mouse_delta = self.world_state.inf.input.mouse_delta();
        self.scripting.set_mouse_delta(mouse_delta.0 as f32, mouse_delta.1 as f32);

        // key flags: bit 0=W, 1=S, 2=A, 3=D, 4=R-Shift (strafe)
        let mut keys: u8 = 0;
        if self.world_state.inf.input.down(InputId::W) { keys |= 1; }
        if self.world_state.inf.input.down(InputId::S) { keys |= 2; }
        if self.world_state.inf.input.down(InputId::A) { keys |= 4; }
        if self.world_state.inf.input.down(InputId::D) { keys |= 8; }
        if self.world_state.inf.input.down(InputId::RShift) { keys |= 16; }
        self.scripting.set_input_keys(keys);

        // Send GAME_START trigger on first frame
        if !self.game_started {
            self.scripting.set_spawn_trigger(scripting::TRIGGER_GAME_START);
            self.game_started = true;
        }

        // Periodic asteroid / mine / powerup spawning is handled in the Itsy script.

        // Run scripting subsystem
        self.scripting.run(&mut self.world, &self.world_state, &mut cmd);

        // Apply scripting commands BEFORE inertia so v_fraction changes take effect
        cmd.run_on(&mut self.world);

        // Shared systems (compute/upgrader/control moved to Itsy)
        system::run_inertia(&mut self.world, &self.world_state);
        system::run_collider(&mut self.world);
        self.render_system.run(&mut self.world, &self.world_state);
        system::run_cleanup(&mut self.world, &self.world_state);

        // render layers (passes created by the Itsy script)
        renderer.fill().texture(&self.background).blendmode(blendmodes::COPY).draw();

        for info in &self.inf.render_layers {
            if let Some(layer) = self.inf.layers.get(info.layer_id as usize) {
                match info.filter {
                    Some(RenderFilter::Bloom) => {
                        renderer.postprocess(&self.bloom, &(), || {
                            renderer.fill().color(Color::alpha_mask(0.3)).draw();
                            renderer.draw_layer(layer, info.component);
                        });
                    }
                    Some(RenderFilter::Glare) => {
                        renderer.postprocess(&self.glare, &blendmodes::SCREEN, || {
                            renderer.fill().color(Color::alpha_mask(0.05)).draw();
                            renderer.draw_layer(layer, info.component);
                        });
                    }
                    None => {
                        renderer.draw_layer(layer, info.component);
                    }
                }
            } else {
                eprintln!("render_layers: invalid layer id {}", info.layer_id);
            }
        }

        for layer in &self.inf.layers {
            layer.clear();
        }
    }
}
