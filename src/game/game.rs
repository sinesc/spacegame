use crate::prelude::*;
use hecs;
use rodio::{MixerDeviceSink, DeviceSinkBuilder};
use rodio::mixer::Mixer;
use crate::scripting::Api;
use crate::timeframe::Timeframe;
use crate::game::system::{RenderLayer, RenderBackground};
use crate::sound::Sound;
use std::collections::HashMap;

mod component;
#[path="system/system.rs"]
mod system;

pub struct Infrastructure {
    pub input: Input,
    /// Radiant context for loading sprites, fonts and textures.
    pub radiant_ctx: Context,
    /// Sprite cache (loaded on first use).
    pub sprite_cache: HashMap<String, Arc<Sprite>>,
    /// Sound cache (loaded on first play).
    pub sound_cache: HashMap<String, Sound>,
    /// Background texture cache (loaded on first draw_background).
    pub background_cache: HashMap<String, Arc<Texture>>,
    pub audio: Mixer,
    /// Audio sink is unused but must stay alive for playback to work.
    pub _audio_sink: MixerDeviceSink,
    /// Render layers created by the Itsy script (`create_layer`); the vector
    /// index is the layer ID shared between Itsy and Rust.
    pub layers: Vec<Arc<Layer>>,
    /// Render passes created by the Itsy script (`add_render_layer`), in draw order.
    pub render_layers: Vec<RenderLayer>,
    /// Background images to show this frame (`draw_background`), in draw order.
    /// Rebuilt by the scripting system each frame (cleared before execution).
    pub background_draws: Vec<RenderBackground>,
    pub font: Arc<Font>,
    pub menu_font: Arc<Font>,
    pub display: Arc<Display>,
    pub monitor: Option<Monitor>,
    /// Layer ID used for Rust-side debug text (set by Itsy via `set_debug_layer`), `u32::MAX` = not set yet. // FIXME: use Option
    pub debug_layer: u32,
}

impl Infrastructure {
    /// The layer designated for Rust-side debug text, if the script set one.
    pub fn debug_layer(&self) -> Option<Arc<Layer>> {
        self.layers.get(self.debug_layer as usize).cloned()
    }
}

pub struct State {
    /// Game time; the Itsy script pauses/resumes it (pause_time / resume_time).
    pub timeframe: Timeframe,
    /// Set by the Itsy script (`request_exit`); checked by the main loop.
    pub exit_requested: bool,
    /// Set by the Itsy script (`request_level_restart`); the main loop rebuilds the level.
    pub restart_requested: bool,
    // track if GAME_START trigger was sent
    pub game_started: bool,
    pub fullscreen: bool,
}

pub struct Game {
    world           : hecs::World,
    render_system   : system::Render,
    scripting       : system::Scripting,
    inf             : Infrastructure,
    state           : State,
}

impl Game {

    pub fn new(input: &Input, display: Arc<Display>, monitor: Option<Monitor>, fullscreen: bool) -> Self {

        let world = hecs::World::new();

        let context = display.context().clone();
        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let menu_font = Font::builder(&context).family("Arial").size(80.0).bold().build().unwrap().arc();
        let mut audio_sink = DeviceSinkBuilder::open_default_sink().unwrap();
        audio_sink.log_on_drop(false);
        let audio = audio_sink.mixer().clone();

        // Render layers are created by the Itsy script (create_layer /
        // add_render_layer); player is spawned via the GAME_START trigger.

        let infrastructure = Infrastructure {
            radiant_ctx         : context.clone(),
            sprite_cache        : HashMap::new(),
            sound_cache         : HashMap::new(),
            background_cache    : HashMap::new(),
            audio               : audio,
            _audio_sink         : audio_sink,
            input               : input.clone(),
            layers              : Vec::new(),
            render_layers       : Vec::new(),
            background_draws    : Vec::new(),
            font                : font,
            menu_font           : menu_font,
            display             : display,
            monitor             : monitor,
            debug_layer         : u32::MAX,
        };

        let state = State {
            timeframe           : Timeframe::new(),
            exit_requested      : false,
            restart_requested   : false,
            fullscreen          : fullscreen,
            game_started        : false,
        };

        Game {
            world           : world,
            render_system   : system::Render::new(context.clone()),
            scripting       : system::Scripting::new(),
            inf             : infrastructure,
            state           : state,
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

    /// Elapsed game time in seconds (read by the main loop for age/delta).
    pub fn game_age(&self) -> f64 {
        Timeframe::duration_to_secs(self.state.timeframe.elapsed())
    }

    /// Current game time rate (0 = paused, 1 = normal).
    pub fn game_rate(&self) -> f64 {
        self.state.timeframe.rate()
    }

    /// True if the Itsy script requested a game exit.
    pub fn exit_requested(&self) -> bool {
        self.state.exit_requested
    }

    /// True if the Itsy script requested a level restart.
    pub fn restart_requested(&self) -> bool {
        self.state.restart_requested
    }

    /// Current fullscreen state (may have been toggled by the Itsy script).
    pub fn is_fullscreen(&self) -> bool {
        self.state.fullscreen
    }

    pub fn process(&mut self, renderer: &Renderer, age: f32, delta: f32) {

        let mut cmd = hecs::CommandBuffer::new();

        // Detect collisions for scripting subsystem
        let collision_pairs = Self::detect_collision_pairs(&self.world);
        self.scripting.set_collisions(collision_pairs);

        // Configure scripting subsystem
        self.scripting.set_game_time(age);

        // Pass mouse position and keyboard input
        let mouse_pos = self.inf.input.mouse();
        self.scripting.set_mouse_pos(mouse_pos.0 as f32, mouse_pos.1 as f32);

        // Pass mouse delta (reliable even when cursor is grabbed/focused)
        let mouse_delta = self.inf.input.mouse_delta();
        self.scripting.set_mouse_delta(mouse_delta.0 as f32, mouse_delta.1 as f32);

        // Input masks: one bit per key (see KEY_* in scripting/mod.rs).
        // `keys` = held down, `pressed` = pressed this frame (incl. repeats),
        // `edge` = initial press this frame (no repeats).
        let (keys, pressed, edge) = {
            let input = &self.inf.input;
            let mut keys = 0u16;
            let mut pressed = 0u16;
            let mut edge = 0u16;
            let mut add_key = |key: InputId, bit: u16| {
                if input.down(key) { keys |= bit; }
                if input.pressed(key, true) { pressed |= bit; }
                if input.pressed(key, false) { edge |= bit; }
            };
            add_key(InputId::W, Api::KEY_W);
            add_key(InputId::S, Api::KEY_S);
            add_key(InputId::A, Api::KEY_A);
            add_key(InputId::D, Api::KEY_D);
            add_key(InputId::RShift, Api::KEY_RSHIFT);
            add_key(InputId::CursorUp, Api::KEY_CURSOR_UP);
            add_key(InputId::CursorDown, Api::KEY_CURSOR_DOWN);
            add_key(InputId::Return, Api::KEY_RETURN);
            add_key(InputId::Escape, Api::KEY_ESCAPE);
            add_key(InputId::Mouse1, Api::KEY_MOUSE1);
            add_key(InputId::LControl, Api::KEY_LCONTROL);
            (keys, pressed, edge)
        };
        self.scripting.set_input_state(keys, pressed, edge);

        // Send GAME_START trigger on first frame
        if !self.state.game_started {
            self.scripting.set_spawn_trigger(Api::TRIGGER_GAME_START);
            self.state.game_started = true;
        }

        // Run scripting subsystem (mutates self.inf via the &mut reference)
        self.scripting.run(&mut self.world, &mut self.inf, &mut self.state, &mut cmd);

        // Apply scripting commands BEFORE inertia so v_fraction changes take effect
        cmd.run_on(&mut self.world);

        // Shared systems (compute/upgrader/control moved to Itsy)
        system::run_inertia(&mut self.world, delta, &self.inf);
        system::run_collider(&mut self.world);
        self.render_system.run(&mut self.world, age, delta, &self.inf, renderer);
        system::run_cleanup(&mut self.world, age);
    }
}