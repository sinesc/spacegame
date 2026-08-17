use crate::prelude::*;
use hecs;
use rodio::{MixerDeviceSink, DeviceSinkBuilder};
use rodio::mixer::Mixer;
use crate::bloom;
use crate::scripting::Api;
use self::system::Scripting;
use crate::timeframe::Timeframe;

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

/// A background image draw requested by the Itsy script (`draw_background`),
/// resolved to a loaded texture. Drawn below all render layers; the image is
/// scaled to cover the display and tiled (wrapped) around the given scroll
/// offset for seamless infinite scrolling.
#[derive(Clone)]
pub struct BackgroundDraw {
    pub texture : Arc<Texture>,
    /// Scroll offset in screen pixels (any value; wrapped to the image size).
    pub offset_x: f32,
    pub offset_y: f32,
}

/// Draws `draw` tiled so it covers the entire display. The image is scaled to
/// cover the display (aspect preserved), then repeated in both directions with
/// the scroll offset wrapped, so any offset scrolls seamlessly.
fn draw_background_tiled(renderer: &Renderer, display_w: f32, display_h: f32, draw: &BackgroundDraw) {
    let (tw, th) = draw.texture.dimensions();
    for (x, y, w, h) in background_tiles(display_w, display_h, tw as f32, th as f32, draw.offset_x, draw.offset_y) {
        renderer.rect(((x, y), (w, h))).texture(&draw.texture).blendmode(blendmodes::COPY).draw();
    }
}

/// Pixel rectangles of the image tiles covering a `display_w` x `display_h`
/// area: the image (scaled to cover the display, aspect preserved) repeated in
/// both directions, shifted by the (wrapped) scroll offset.
fn background_tiles(display_w: f32, display_h: f32, img_w: f32, img_h: f32, offset_x: f32, offset_y: f32) -> Vec<(f32, f32, f32, f32)> {
    let scale = (display_w / img_w).max(display_h / img_h);
    let iw = img_w * scale;
    let ih = img_h * scale;
    let ox = wrap_pos(offset_x, iw);
    let oy = wrap_pos(offset_y, ih);
    // The cover scale guarantees iw >= display_w and ih >= display_h, so at
    // most 2 tiles per axis are needed: a second one exists only when the
    // offset pushes the first tile's right/bottom edge past the display.
    // Tile k sits at k * size - offset.
    let kx_max = if ox + display_w > iw { 1 } else { 0 };
    let ky_max = if oy + display_h > ih { 1 } else { 0 };
    let mut tiles = Vec::new();
    for ky in 0..=ky_max {
        let y = ky as f32 * ih - oy;
        for kx in 0..=kx_max {
            let x = kx as f32 * iw - ox;
            tiles.push((x, y, iw, ih));
        }
    }
    tiles
}

/// Wraps `v` into [0, period) (Rust's `%` may return negative values).
fn wrap_pos(v: f32, period: f32) -> f32 {
    let w = v % period;
    if w < 0.0 { w + period } else { w }
}

pub struct Infrastructure {
    pub input     : Input,
    pub audio     : Mixer,
    /// Render layers created by the Itsy script (`create_layer`); the vector
    /// index is the layer ID shared between Itsy and Rust.
    pub layers    : Vec<Arc<Layer>>,
    /// Render passes created by the Itsy script (`add_render_layer`), in draw order.
    pub render_layers : Vec<RenderLayer>,
    /// Background images to show this frame (`draw_background`), in draw order.
    /// Rebuilt by the scripting system each frame (cleared before execution).
    pub background_draws : Vec<BackgroundDraw>,
    pub font      : Arc<Font>,
    /// Font for Itsy-side menu text (Arial 80 bold).
    pub menu_font : Arc<Font>,
    /// Game time; the Itsy script pauses/resumes it (pause_time / resume_time).
    pub timeframe : Timeframe,
    /// Set by the Itsy script (`request_exit`); checked by the main loop.
    pub exit_requested    : bool,
    /// Set by the Itsy script (`request_level_restart`); the main loop rebuilds the level.
    pub restart_requested : bool,
    /// Display handle for the `toggle_fullscreen` API.
    pub display   : Arc<Display>,
    pub monitor   : Option<Monitor>,
    pub fullscreen: bool,
    /// Layer ID used for Rust-side debug text (set by Itsy via `set_debug_layer`);
    /// `u32::MAX` = not set yet.
    pub debug_layer : u32,
}

impl Infrastructure {
    /// The layer designated for Rust-side debug text, if the script set one.
    pub fn debug_layer(&self) -> Option<Arc<Layer>> {
        self.layers.get(self.debug_layer as usize).cloned()
    }
}

pub struct Level {
    world           : hecs::World,
    render_system   : system::Render,
    inf             : Infrastructure,
    _audio_sink     : MixerDeviceSink,
    bloom           : postprocessors::Bloom,
    glare           : bloom::Bloom,
    scripting       : Scripting,
    game_started    : bool,  // track if GAME_START trigger was sent
}

impl Level {

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
            audio             : audio,
            input             : input.clone(),
            layers            : Vec::new(),
            render_layers     : Vec::new(),
            background_draws  : Vec::new(),
            font              : font,
            menu_font         : menu_font,
            timeframe         : Timeframe::new(),
            exit_requested    : false,
            restart_requested : false,
            display           : display,
            monitor           : monitor,
            fullscreen        : fullscreen,
            debug_layer       : u32::MAX,
        };

        let mut bloom = postprocessors::Bloom::new(&context, (1920u32, 1080u32), 2);
        bloom.clear = false;
        bloom.draw_color = Color::alpha_pm(0.15);

        Level {
            world           : world,
            render_system   : system::Render::new(),
            _audio_sink     : audio_sink,
            bloom           : bloom,
            glare           : bloom::Bloom::new(&context, (1920, 1080), 2, 5, 5.0),
            inf             : infrastructure,
            scripting       : Scripting::new(context.clone()),
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

    /// Elapsed game time in seconds (read by the main loop for age/delta).
    pub fn game_age(&self) -> f64 {
        Timeframe::duration_to_secs(self.inf.timeframe.elapsed())
    }

    /// Current game time rate (0 = paused, 1 = normal).
    pub fn game_rate(&self) -> f64 {
        self.inf.timeframe.rate()
    }

    /// True if the Itsy script requested a game exit.
    pub fn exit_requested(&self) -> bool {
        self.inf.exit_requested
    }

    /// True if the Itsy script requested a level restart.
    pub fn restart_requested(&self) -> bool {
        self.inf.restart_requested
    }

    /// Current fullscreen state (may have been toggled by the Itsy script).
    pub fn is_fullscreen(&self) -> bool {
        self.inf.fullscreen
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
        if !self.game_started {
            self.scripting.set_spawn_trigger(Api::TRIGGER_GAME_START);
            self.game_started = true;
        }

        // Periodic asteroid / mine / powerup spawning is handled in the Itsy script.

        // Run scripting subsystem (mutates self.inf via the &mut reference)
        self.scripting.run(&mut self.world, &mut self.inf, &mut cmd);

        // Apply scripting commands BEFORE inertia so v_fraction changes take effect
        cmd.run_on(&mut self.world);

        // Shared systems (compute/upgrader/control moved to Itsy)
        system::run_inertia(&mut self.world, delta, &self.inf);
        system::run_collider(&mut self.world);
        self.render_system.run(&mut self.world, age, delta, &self.inf);
        system::run_cleanup(&mut self.world, age);

        // Backgrounds requested by the Itsy script (draw_background):
        // tiled below all render layers, wrapped for infinite scrolling.
        let (display_w, display_h) = self.inf.display.dimensions();
        for draw in self.inf.background_draws.iter() {
            draw_background_tiled(renderer, display_w as f32, display_h as f32, draw);
        }

        // render layers (passes created by the Itsy script)
        for info in self.inf.render_layers.iter() {
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

        for layer in self.inf.layers.iter() {
            layer.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks that every tile is fully contained in a plausible region and
    /// that the union of tiles covers the whole display.
    fn assert_covers(tiles: &[(f32, f32, f32, f32)], display_w: f32, display_h: f32) {
        assert!(!tiles.is_empty());
        // Cover in x: [min_start, max_end] must contain [0, display_w].
        let min_x = tiles.iter().map(|t| t.0).fold(f32::MAX, f32::min);
        let max_x = tiles.iter().map(|t| t.0 + t.2).fold(f32::MIN, f32::max);
        let min_y = tiles.iter().map(|t| t.1).fold(f32::MAX, f32::min);
        let max_y = tiles.iter().map(|t| t.1 + t.3).fold(f32::MIN, f32::max);
        assert!(min_x <= 0.0, "gap at left edge: min_x = {}", min_x);
        assert!(max_x >= display_w, "gap at right edge: max_x = {}", max_x);
        assert!(min_y <= 0.0, "gap at top edge: min_y = {}", min_y);
        assert!(max_y >= display_h, "gap at bottom edge: max_y = {}", max_y);
    }

    #[test]
    fn wrap_pos_wraps_into_range() {
        assert!((wrap_pos(0.0, 10.0) - 0.0).abs() < 1e-6);
        assert!((wrap_pos(10.0, 10.0) - 0.0).abs() < 1e-6);
        assert!((wrap_pos(25.5, 10.0) - 5.5).abs() < 1e-6);
        assert!((wrap_pos(-5.0, 10.0) - 5.0).abs() < 1e-6);
        assert!((wrap_pos(-15.25, 10.0) - 4.75).abs() < 1e-6);
    }

    #[test]
    fn tiles_cover_display_at_offset_zero() {
        // blue.jpg: 3200x1000 on a 1920x1080 display.
        let tiles = background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 0.0, 0.0);
        assert_covers(&tiles, 1920.0, 1080.0);
        // Cover scale = 1080/1000 = 1.08 -> 3456x1080: exactly one tile.
        assert_eq!(tiles.len(), 1);
        assert!((tiles[0].0 - 0.0).abs() < 1e-3 && (tiles[0].1 - 0.0).abs() < 1e-3);
        assert!((tiles[0].2 - 3456.0).abs() < 0.01 && (tiles[0].3 - 1080.0).abs() < 0.01);
    }

    #[test]
    fn tiles_cover_display_when_scrolled() {
        // A mid-scroll offset needs a second, shifted tile pair.
        let tiles = background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 1500.0, 200.0);
        assert_covers(&tiles, 1920.0, 1080.0);
        assert!(tiles.len() <= 4);
    }

    #[test]
    fn tiles_wrap_unbounded_offsets() {
        // Any offset (negative, unbounded) must give the same tiling as its
        // wrapped equivalent: seamless infinite scrolling.
        let a = background_tiles(1920.0, 1080.0, 3200.0, 1000.0, -42.5, 0.0);
        let b = background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 3456.0 - 42.5, 0.0);
        assert_eq!(a.len(), b.len());
        for (ta, tb) in a.iter().zip(b.iter()) {
            assert!((ta.0 - tb.0).abs() < 1e-3 && (ta.1 - tb.1).abs() < 1e-3);
        }
        let c = background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 100_000.0, -7.0);
        assert_covers(&c, 1920.0, 1080.0);
    }

    #[test]
    fn tiles_cover_display_for_small_images() {
        // An image smaller than the display (in one axis) still covers it:
        // square 100x100 -> scaled to 1920x1920 (cover), 2 rows needed.
        let tiles = background_tiles(1920.0, 1080.0, 100.0, 100.0, 300.0, 600.0);
        assert_covers(&tiles, 1920.0, 1080.0);
        assert!(tiles.len() <= 4);
    }
}
