mod context;

pub use self::context::{ScriptContext, EntityData, ApiOp, SpawnRequest};

use crate::prelude::*;
use itsy;

// Define the Itsy API type.
//
// The Rust<->Itsy protocol constants (key bits, entity types, spawn triggers,
// motion/blend/filter encodings and the "no layer" sentinel) are declared once,
// before any function, and become associated constants on `Api`. Rust code
// refers to them as `Api::KEY_W`; the Itsy script refers to the same definition
// (as `Api::KEY_W`, or bare `KEY_W` after `use Api::KEY_W`). This makes the
// macro the single source of truth, so the two sides can never drift.
itsy::itsy_api! {
    pub Api<ScriptContext> {
        // Input key bits (the masks passed to Itsy each frame via set_input_state).
        const KEY_W           : u16 = 1;
        const KEY_S           : u16 = 2;
        const KEY_A           : u16 = 4;
        const KEY_D           : u16 = 8;
        const KEY_RSHIFT      : u16 = 16;
        const KEY_CURSOR_UP   : u16 = 32;
        const KEY_CURSOR_DOWN : u16 = 64;
        const KEY_RETURN      : u16 = 128;
        const KEY_ESCAPE      : u16 = 256;
        const KEY_MOUSE1      : u16 = 512;
        const KEY_LCONTROL    : u16 = 1024;

        // Entity type IDs.
        const ET_NONE         : u16 = 0;
        const ET_PLAYER       : u16 = 1;
        const ET_ASTEROID     : u16 = 2;
        const ET_MINE_RED     : u16 = 3;
        const ET_MINE_GREEN   : u16 = 4;
        const ET_POWERUP_DUAL : u16 = 5;
        const ET_POWERUP_TRIPLE : u16 = 6;
        const ET_PROJECTILE   : u16 = 7;
        const ET_EXPLOSION    : u16 = 8;

        // Spawn triggers (Rust -> Itsy).
        const TRIGGER_NONE      : u32 = 0;
        const TRIGGER_GAME_START: u32 = 4;

        // Inertial motion types (set_v_motion; order matches component::InertialMotionType).
        const MOTION_CONST    : u32 = 0;
        const MOTION_FOLLOW   : u32 = 1;  // move + face movement direction
        const MOTION_STRAFE   : u32 = 2;  // move, keep current angle
        const MOTION_DETACHED : u32 = 3;  // rotate toward v_fraction, no movement

        // Layer blend modes (create_layer).
        const BLEND_NORMAL  : u32 = 0;
        const BLEND_ADD     : u32 = 1;
        const BLEND_LIGHTEN : u32 = 2;

        // Render pass filters (add_render_layer).
        const FILTER_NONE  : u32 = 0;
        const FILTER_BLOOM : u32 = 1;
        const FILTER_GLARE : u32 = 2;

        // "No layer" sentinel for layer / effect_layer IDs (Rust treats it as absent).
        const LAYER_ID_NONE : u32 = u32::MAX;

        fn get_hitpoints(&mut context, id: u64) -> f32 {
            context.entity_data.get(&id).map(|e| e.hitpoints).unwrap_or(0.0)
        }
        fn get_position_x(&mut context, id: u64) -> f32 {
            context.entity_data.get(&id).map(|e| e.position.0).unwrap_or(0.0)
        }
        fn get_position_y(&mut context, id: u64) -> f32 {
            context.entity_data.get(&id).map(|e| e.position.1).unwrap_or(0.0)
        }
        fn get_velocity_x(&mut context, id: u64) -> f32 {
            context.entity_data.get(&id).map(|e| e.velocity.0).unwrap_or(0.0)
        }
        fn get_velocity_y(&mut context, id: u64) -> f32 {
            context.entity_data.get(&id).map(|e| e.velocity.1).unwrap_or(0.0)
        }
        fn get_angle(&mut context, id: u64) -> f32 {
            context.entity_data.get(&id).map(|e| e.angle).unwrap_or(0.0)
        }
        fn get_script_type(&mut context, id: u64) -> u16 {
            context.entity_data.get(&id).map(|e| e.script_type).unwrap_or(0)
        }
        fn get_faction(&mut context, id: u64) -> u16 {
            context.entity_data.get(&id).map(|e| e.faction).unwrap_or(0)
        }
        fn is_alive(&mut context, id: u64) -> bool {
            context.entity_data.get(&id).map(|e| e.alive).unwrap_or(false)
        }
        fn get_think_count(&mut context) -> i32 {
            context.think_entities.len() as i32
        }
        fn get_think_id(&mut context, index: u32) -> u64 {
            context.think_entities.get(index as usize).copied().unwrap_or(0)
        }
        fn get_collision_count(&mut context) -> i32 {
            (context.collisions.len() / 2) as i32
        }
        fn get_collision_id(&mut context, index: u32) -> u64 {
            context.collisions.get(index as usize).copied().unwrap_or(0)
        }
        fn get_game_time(&mut context) -> f32 {
            context.game_time
        }
        fn get_mouse_x(&mut context) -> f32 {
            context.mouse_pos.0
        }
        fn get_mouse_y(&mut context) -> f32 {
            context.mouse_pos.1
        }
        fn get_mouse_delta_x(&mut context) -> f32 {
            context.mouse_delta.0
        }
        fn get_mouse_delta_y(&mut context) -> f32 {
            context.mouse_delta.1
        }
        /// Keys currently held down (level-triggered `down` semantics).
        fn get_input_keys(&mut context) -> u16 {
            context.input_keys
        }
        /// Keys pressed this frame, including repeat events while held
        /// (edge-triggered `pressed(key, true)` semantics).
        fn get_input_pressed(&mut context) -> u16 {
            context.input_pressed
        }
        /// Keys pressed this frame, initial press only (no repeats).
        fn get_input_edge(&mut context) -> u16 {
            context.input_edge
        }
        fn get_dying_count(&mut context) -> i32 {
            context.dying_entities.len() as i32
        }
        fn get_dying_id(&mut context, index: u32) -> u64 {
            context.dying_entities.get(index as usize).copied().unwrap_or(0)
        }
        fn get_spawn_trigger(&mut context) -> u32 {
            let t = context.spawn_trigger;
            context.spawn_trigger = 0;  // consume trigger
            t
        }
        fn get_rand_range(&mut context, min: f32, max: f32) -> f32 {
            context.rng.range(min, max)
        }
        /// All sprite file paths (recursive listing of res/sprite, sorted).
        /// Generated once on the Rust side; the returned vector index is the
        /// sprite ID shared between Itsy and Rust.
        fn get_sprites(&mut context) -> [ String ] {
            context.sprite_list.clone()
        }
        /// All sound file paths (recursive listing of res/sound, sorted).
        /// Generated once on the Rust side; the returned vector index is the
        /// sound ID shared between Itsy and Rust.
        /// The script groups files into sound effects itself.
        fn get_sounds(&mut context) -> [ String ] {
            context.sound_list.clone()
        }
        /// All background image file paths (recursive listing of res/background,
        /// sorted). Generated once on the Rust side; the returned vector index
        /// is the background ID shared between Itsy and Rust.
        fn get_backgrounds(&mut context) -> [ String ] {
            context.background_list.clone()
        }
        /// Create a new render layer and return its ID (vector index into the
        /// layer list, shared between Itsy and Rust).
        /// Layer size = (scale * 1920) x (scale * 1080).
        /// `blendmode`: 0 = normal, 1 = add, 2 = lighten.
        fn create_layer(&mut context, scale: f32, blendmode: u32) -> u32 {
            // The ID is the layers-vector index the executor will assign
            // (FIFO; only CreateLayer ops append to the vector).
            let id = context.next_layer_id;
            context.next_layer_id += 1;
            context.pending.push(ApiOp::CreateLayer { scale, blend: blendmode });
            id
        }
        /// Register a render pass for a layer (in draw order), mirroring the old
        /// layer.yaml "render" section.
        /// `filter`: 0 = none, 1 = bloom, 2 = glare. `component` = z-order
        /// within the layer.
        fn add_render_layer(&mut context, layer_id: u32, filter: u32, component: u32) {
            context.pending.push(ApiOp::AddRenderLayer { layer_id, filter, component });
        }
        /// Draw text in white (alpha 0..=1) on a layer.
        fn write_text(&mut context, layer_id: u32, msg: String, x: f32, y: f32, alpha: f32) {
            context.pending.push(ApiOp::WriteText { layer_id, msg, x, y, alpha, menu: false });
        }
        /// Tell Rust which layer to use for its own debug text output.
        fn set_debug_layer(&mut context, layer_id: u32) {
            context.pending.push(ApiOp::SetDebugLayer(layer_id));
        }
        /// Draw menu text (80 px bold font) in white (alpha 0..=1) on a layer.
        fn write_menu_text(&mut context, layer_id: u32, msg: String, x: f32, y: f32, alpha: f32) {
            context.pending.push(ApiOp::WriteText { layer_id, msg, x, y, alpha, menu: true });
        }
        /// Lerp the game time rate to 0 over 500 ms (pause, e.g. when the menu opens).
        fn pause_time(&mut context) {
            context.pending.push(ApiOp::PauseTime);
        }
        /// Lerp the game time rate back to 1 over 500 ms (resume).
        fn resume_time(&mut context) {
            context.pending.push(ApiOp::ResumeTime);
        }
        /// Ask the main loop to exit the game.
        fn request_exit(&mut context) {
            context.pending.push(ApiOp::RequestExit);
        }
        /// Ask the main loop to rebuild the level (fresh VM, layers, game time).
        fn request_level_restart(&mut context) {
            context.pending.push(ApiOp::RequestLevelRestart);
        }
        /// Toggle windowed/fullscreen mode (starts in the mode the game was launched with).
        fn toggle_fullscreen(&mut context) {
            context.pending.push(ApiOp::ToggleFullscreen);
        }
        /// Play a sound file by ID (index into `get_sounds()`).
        /// Files are loaded on first use and cached.
        fn play_sound(&mut context, id: u32) {
            if (id as usize) >= context.sound_list.len() {
                eprintln!("play_sound: invalid id {}", id);
                return;
            }
            context.pending.push(ApiOp::PlaySound { id });
        }
        /// Show a background image (index into `get_backgrounds()`) this frame,
        /// scrolling it by `offset_x` / `offset_y` screen pixels.
        /// The image is scaled to cover the display (aspect preserved) and
        /// wraps around, so any offset (incl. negative / unbounded) gives
        /// seamless infinite scrolling; increasing `offset_x` moves the image
        /// left (camera moves right, like in a right-scrolling side-scroller).
        /// Backgrounds are drawn below all render layers, in call order.
        /// Call once per frame for every background that should be visible.
        fn draw_background(&mut context, id: u32, offset_x: f32, offset_y: f32) {
            if (id as usize) >= context.background_list.len() {
                eprintln!("draw_background: invalid id {}", id);
                return;
            }
            context.pending.push(ApiOp::DrawBackground { id, offset_x, offset_y });
        }
        fn debug_print(&mut _context, msg: String) {
            eprintln!("ITSY: {}", msg);
        }

        // --- Action API ---
        // API calls record operations in `context.pending`; the Scripting
        // system executes them after vm.run() (request queue — this is what
        // keeps the API free of raw pointers to the world/infrastructure).

        /// `fade` = number of seconds the entity's visual fades out over, at the end
        /// of its lifetime (0 = no fading).
        /// `fps` = sprite animation speed (0 = no animation; frame is then picked
        /// from the entity's lean, like the original sprite-variant behavior).
        /// `sprite_id` / `layer_id` / `effect_layer_id` index into `get_sprites()` /
        /// `get_layers()`; `u32::MAX` as a layer ID means "no layer".
        /// `color_r/g/b` tint the sprite (alpha is always 1.0; values may exceed 1.0
        /// on additive layers).
        fn spawn_entity(&mut context, entity_type: u16, sprite_id: u32, layer_id: u32, effect_layer_id: u32, px: f32, py: f32, angle: f32, vx: f32, vy: f32, faction: u16, hitpoints: f32, radius: f32, lifetime: f32, fade: f32, fps: u32, color_r: f32, color_g: f32, color_b: f32) {
            context.pending.push(ApiOp::Spawn(SpawnRequest {
                entity_type, sprite_id, layer_id, effect_layer_id,
                px, py, angle, vx, vy, faction,
                hitpoints, radius, lifetime, fade, fps,
                color_r, color_g, color_b,
                game_time: context.game_time,
            }));
        }
        fn destroy_entity(&mut context, entity_id: u64) {
            context.pending.push(ApiOp::Despawn(entity_id));
        }
        /// Set v_fraction and motion type. Motion type values match the order of
        /// component::InertialMotionType: 0=Const, 1=FollowVector (move + face
        /// movement), 2=StrafeVector (move, keep angle), 3=Detached (rotate only).
        fn set_v_motion(&mut context, entity_id: u64, motion_type: u32, vx: f32, vy: f32) {
            context.pending.push(ApiOp::SetVMotion { id: entity_id, motion: motion_type, vx, vy });
        }
        fn set_angle(&mut context, entity_id: u64, angle: f32) {
            context.pending.push(ApiOp::SetAngle { id: entity_id, angle });
        }
        fn set_hitpoints(&mut context, entity_id: u64, hp: f32) {
            context.pending.push(ApiOp::SetHitpoints { id: entity_id, hp });
        }
        fn apply_damage(&mut context, entity_id: u64, damage: f32) {
            context.pending.push(ApiOp::ApplyDamage { id: entity_id, damage });
        }
    }
}
