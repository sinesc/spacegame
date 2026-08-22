use crate::prelude::*;
use radiant_utils::maths::Mat4;
use hecs;
use rodio::mixer::Mixer;
use crate::timeframe::Timeframe;
use crate::game::system::{RenderLayer, RenderBackground};
use crate::sound::Sound;
use std::collections::HashMap;

mod component;
#[path="system/system.rs"]
mod system;

pub struct Infrastructure {
    pub input: Input,
    /// Sprite cache (loaded on first use).
    pub sprite_cache: HashMap<String, Arc<Sprite>>,
    /// Sound cache (loaded on first play).
    pub sound_cache: HashMap<String, Sound>,
    /// Background texture cache (loaded on first draw_background).
    pub background_cache: HashMap<String, Arc<Texture>>,
    pub audio: Mixer,
    /// Render layers created by the Itsy script (`create_layer`); the vector
    /// index is the layer ID shared between Itsy and Rust.
    pub layers: Vec<Arc<Layer>>,
    /// The `create_layer` scale of each layer in `layers` (parallel vector;
    /// needed to re-apply the layer view matrix on a display resize).
    pub layer_scales: Vec<f32>,
    /// Render passes created by the Itsy script (`add_render_layer`), in draw order.
    pub render_layers: Vec<RenderLayer>,
    /// Background images to show this frame (`draw_background`), in draw order.
    /// Rebuilt by the scripting system each frame (cleared before execution).
    pub background_draws: Vec<RenderBackground>,
    pub font: Arc<Font>,
    pub menu_font: Arc<Font>,
    pub display: Arc<Display>,
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
    /// Set by the Itsy script (`set_resolution`); the main loop applies it
    /// after swap_frame (Option so `take_resolution_request` can consume it).
    pub resolution_requested: Option<(u32, u32)>,
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

    pub fn new(input: &Input, display: Arc<Display>, fullscreen: bool, audio: &Mixer) -> Self {

        let world = hecs::World::new();
        let context = display.context().clone();
        let font = Font::builder(&context).family("Arial").size(20.0).build().unwrap().arc();
        let menu_font = Font::builder(&context).family("Arial").size(80.0).bold().build().unwrap().arc();

        let dimensions = display.dimensions();

        let infrastructure = Infrastructure {
            sprite_cache        : HashMap::new(),
            sound_cache         : HashMap::new(),
            background_cache    : HashMap::new(),
            audio               : audio.clone(),
            input               : input.clone(),
            layers              : Vec::new(),
            layer_scales        : Vec::new(),
            render_layers       : Vec::new(),
            background_draws    : Vec::new(),
            font                : font,
            menu_font           : menu_font,
            display             : display,
            debug_layer         : u32::MAX,
        };

        let state = State {
            timeframe           : Timeframe::new(),
            exit_requested      : false,
            restart_requested   : false,
            resolution_requested: None,
            fullscreen          : fullscreen,
        };

        Game {
            world           : world,
            render_system   : system::Render::new(&context, dimensions),
            scripting       : system::Scripting::new(),
            inf             : infrastructure,
            state           : state,
        }
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

    /// Consume a pending resolution change requested by the Itsy script.
    pub fn take_resolution_request(&mut self) -> Option<(u32, u32)> {
        std::mem::take(&mut self.state.resolution_requested)
    }

    /// Apply a live display resize (called by the main loop after swap_frame,
    /// when no frame is prepared). Re-sets the script layers' view matrices to
    /// the new size (keeping their scales) and rebuilds the postprocessors.
    /// If the game is in fullscreen, it drops to windowed first: the window is
    /// locked to the monitor size in fullscreen, so the resize would be a no-op.
    pub fn apply_resolution(&mut self, width: u32, height: u32) {
        let (before_w, before_h) = self.inf.display.dimensions();
        eprintln!("[debug] apply_resolution: requested ({width}, {height}), before = ({before_w}, {before_h}), state.fullscreen = {}", self.state.fullscreen);
        if self.state.fullscreen {
            self.inf.display.set_windowed();
            self.state.fullscreen = false;
            eprintln!("[debug] apply_resolution: dropped to windowed");
        }
        self.inf.display.set_dimensions((width, height));
        // Use the size the window actually got (the compositor may clamp it).
        // NOTE: winit may report the old size here — the WM resize event can
        // arrive later, so a stale value is not proof the resize failed.
        let (w, h) = self.inf.display.dimensions();
        eprintln!("[debug] apply_resolution: display.dimensions() after set_dimensions = ({w}, {h})");
        for (layer, scale) in self.inf.layers.iter().zip(self.inf.layer_scales.iter()) {
            // Mat4::viewport() returns the radiant_utils wrapper; .0 is the raw
            // [[f32;4];4] that Layer::set_view_matrix expects.
            let matrix = Mat4::viewport(*scale * w as f32, *scale * h as f32);
            layer.set_view_matrix(matrix.0);
        }
        self.render_system.resize(&self.inf.display.context(), (w, h));
    }

    /// Process a game frame.
    pub fn process(&mut self, renderer: &Renderer, age: f32, delta: f32) {

        // Run scripting subsystem and apply script commands
        self.scripting.prepare_frame(&mut self.world, &mut self.inf, age);
        let mut cmd = hecs::CommandBuffer::new();
        self.scripting.run(&mut self.world, &mut self.inf, &mut self.state, &mut cmd);
        cmd.run_on(&mut self.world);

        // Shared systems
        system::run_inertia(&mut self.world, delta, &self.inf);
        system::run_collider(&mut self.world);
        self.render_system.run(&mut self.world, age, delta, &self.inf, renderer);
        system::run_cleanup(&mut self.world, age);
    }
}