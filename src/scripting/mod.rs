mod context;

pub use self::context::{ScriptContext, EntityData};

use crate::prelude::*;
use crate::level::component;
use crate::level::{RenderLayer, RenderFilter};
use crate::sound::Sound;
use hecs;
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
        /// Create a new render layer and return its ID (vector index into the
        /// layer list, shared between Itsy and Rust).
        /// Layer size = (scale * 1920) x (scale * 1080).
        /// `blendmode`: 0 = normal, 1 = add, 2 = lighten.
        fn create_layer(&mut context, scale: f32, blendmode: u32) -> u32 {
            let inf = unsafe { &mut *context.infrastructure };
            let layer = Layer::new((scale * 1920., scale * 1080.)).arc();
            match blendmode {
                Api::BLEND_ADD     => { layer.set_blendmode(blendmodes::ADD); }
                Api::BLEND_LIGHTEN => { layer.set_blendmode(blendmodes::LIGHTEN); }
                _ => {}
            }
            let id = inf.layers.len() as u32;
            inf.layers.push(layer);
            id
        }
        /// Register a render pass for a layer (in draw order), mirroring the old
        /// layer.yaml "render" section.
        /// `filter`: 0 = none, 1 = bloom, 2 = glare. `component` = z-order
        /// within the layer.
        fn add_render_layer(&mut context, layer_id: u32, filter: u32, component: u32) {
            let inf = unsafe { &mut *context.infrastructure };
            inf.render_layers.push(RenderLayer {
                layer_id,
                filter: match filter {
                    Api::FILTER_BLOOM => Some(RenderFilter::Bloom),
                    Api::FILTER_GLARE => Some(RenderFilter::Glare),
                    _ => None,
                },
                component,
            });
        }
        /// Draw text in white (alpha 0..=1) on a layer.
        fn write_text(&mut context, layer_id: u32, msg: String, x: f32, y: f32, alpha: f32) {
            let inf = unsafe { &mut *context.infrastructure };
            if let Some(layer) = inf.layers.get(layer_id as usize) {
                inf.font.write(layer, &msg, (x, y), Color::alpha_pm(alpha));
            }
        }
        /// Tell Rust which layer to use for its own debug text output.
        fn set_debug_layer(&mut context, layer_id: u32) {
            let inf = unsafe { &mut *context.infrastructure };
            inf.debug_layer.store(layer_id, std::sync::atomic::Ordering::Relaxed);
        }
        /// Draw menu text (80 px bold font) in white (alpha 0..=1) on a layer.
        fn write_menu_text(&mut context, layer_id: u32, msg: String, x: f32, y: f32, alpha: f32) {
            let inf = unsafe { &*context.infrastructure };
            if let Some(layer) = inf.layers.get(layer_id as usize) {
                inf.menu_font.write(layer, &msg, (x, y), Color::alpha_pm(alpha));
            }
        }
        /// Lerp the game time rate to 0 over 500 ms (pause, e.g. when the menu opens).
        fn pause_time(&mut context) {
            let inf = unsafe { &mut *context.infrastructure };
            inf.timeframe.lerp_rate(0.0, Duration::from_millis(500));
        }
        /// Lerp the game time rate back to 1 over 500 ms (resume).
        fn resume_time(&mut context) {
            let inf = unsafe { &mut *context.infrastructure };
            inf.timeframe.lerp_rate(1.0, Duration::from_millis(500));
        }
        /// Ask the main loop to exit the game.
        fn request_exit(&mut context) {
            let inf = unsafe { &mut *context.infrastructure };
            inf.exit_requested.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        /// Ask the main loop to rebuild the level (fresh VM, layers, game time).
        fn request_level_restart(&mut context) {
            let inf = unsafe { &mut *context.infrastructure };
            inf.restart_requested.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        /// Toggle windowed/fullscreen mode (starts in the mode the game was launched with).
        fn toggle_fullscreen(&mut context) {
            let inf = unsafe { &mut *context.infrastructure };
            if inf.fullscreen.load(std::sync::atomic::Ordering::Relaxed) {
                inf.display.set_windowed();
                inf.fullscreen.store(false, std::sync::atomic::Ordering::Relaxed);
            } else if let Some(monitor) = &inf.monitor {
                inf.display.set_fullscreen(Some(monitor.clone())).unwrap();
                inf.fullscreen.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        /// Play a sound file by ID (index into `get_sounds()`).
        /// Files are loaded on first use and cached.
        fn play_sound(&mut context, id: u32) {
            let id = id as usize;
            if id >= context.sound_list.len() {
                eprintln!("play_sound: invalid id {}", id);
                return;
            }
            let name = context.sound_list[id].clone();
            let inf = unsafe { &*context.infrastructure };
            let cache = unsafe { &mut *context.sound_cache };
            if cache.get(&name).is_none() {
                match Sound::load(&name) {
                    Ok(sound) => { cache.insert(name.clone(), sound); }
                    Err(e) => { eprintln!("play_sound: failed to load '{}': {}", name, e); return; }
                }
            }
            let sound = cache.get(&name).unwrap();
            inf.audio.add(sound.decoder());
        }
        fn debug_print(&mut _context, msg: String) {
            eprintln!("ITSY: {}", msg);
        }

        // --- Direct action API (replaces command buffer) ---

        /// `fade` = number of seconds the entity's visual fades out over, at the end
        /// of its lifetime (0 = no fading).
        /// `fps` = sprite animation speed (0 = no animation; frame is then picked
        /// from the entity's lean, like the original sprite-variant behavior).
        /// `sprite_id` / `layer_id` / `effect_layer_id` index into `get_sprites()` /
        /// `get_layers()`; `u32::MAX` as a layer ID means "no layer".
        /// `color_r/g/b` tint the sprite (alpha is always 1.0; values may exceed 1.0
        /// on additive layers).
        fn spawn_entity(&mut context, entity_type: u16, sprite_id: u32, layer_id: u32, effect_layer_id: u32, px: f32, py: f32, angle: f32, vx: f32, vy: f32, faction: u16, hitpoints: f32, radius: f32, lifetime: f32, fade: f32, fps: u32, color_r: f32, color_g: f32, color_b: f32) {
            spawn_entity_from_context(&context, entity_type, sprite_id, layer_id, effect_layer_id, px, py, angle, vx, vy, faction, hitpoints, radius, lifetime, fade, fps, color_r, color_g, color_b);
        }
        fn destroy_entity(&mut context, entity_id: u64) {
            let world = unsafe { &mut *context.world };
            if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                if let Err(e) = world.despawn(entity) {
                    eprintln!("destroy_entity(id={}) failed: {:?}", entity_id, e);
                }
            }
        }
        /// Set v_fraction and motion type. Motion type values match the order of
        /// component::InertialMotionType: 0=Const, 1=FollowVector (move + face
        /// movement), 2=StrafeVector (move, keep angle), 3=Detached (rotate only).
        fn set_v_motion(&mut context, entity_id: u64, motion_type: u32, vx: f32, vy: f32) {
            let world = unsafe { &mut *context.world };
            if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                if let Ok(mut inertial) = world.get::<&mut component::Inertial>(entity) {
                    inertial.v_fraction = Vec2(vx, vy);
                    inertial.motion_type = match motion_type {
                        Api::MOTION_CONST  => component::InertialMotionType::Const,
                        Api::MOTION_FOLLOW => component::InertialMotionType::FollowVector,
                        Api::MOTION_STRAFE => component::InertialMotionType::StrafeVector,
                        _ => component::InertialMotionType::Detached,
                    };
                }
            }
        }
        fn set_angle(&mut context, entity_id: u64, angle: f32) {
            let world = unsafe { &mut *context.world };
            if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                if let Ok(mut spatial) = world.get::<&mut component::Spatial>(entity) {
                    spatial.angle = Angle(angle);
                }
            }
        }
        fn set_hitpoints(&mut context, entity_id: u64, hp: f32) {
            let world = unsafe { &mut *context.world };
            if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                if let Ok(mut hitpoints) = world.get::<&mut component::Hitpoints>(entity) {
                    hitpoints.0 = hp;
                }
            }
        }
        fn apply_damage(&mut context, entity_id: u64, damage: f32) {
            let world = unsafe { &mut *context.world };
            if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                if let Ok(mut hitpoints) = world.get::<&mut component::Hitpoints>(entity) {
                    hitpoints.0 -= damage;
                }
            }
        }
    }
}

/// Create an ECS entity from Itsy spawn API call (free function for API bridge).
/// `vx, vy` seed the entity's initial velocity (Const motion applies v_current * delta
/// each frame, so this is also how entities get their drift speed).
/// Visuals (sprite / layers / color) are passed by the script as IDs into
/// `ScriptContext::sprite_list` / the Infrastructure layer list; `u32::MAX` = no layer.
/// `entity_type` only drives the Script component, player speed and the
/// explosion no-move / no-collision rules.
fn spawn_entity_from_context(ctx: &ScriptContext,
    entity_type: u16, sprite_id: u32, layer_id: u32, effect_layer_id: u32,
    px: f32, py: f32, angle: f32, vx: f32, vy: f32, faction: u16,
    hitpoints: f32, radius: f32, lifetime: f32, fade: f32, fps: u32,
    color_r: f32, color_g: f32, color_b: f32) {
    let cmd = unsafe { &mut *ctx.cmd };
    let radiant_ctx = unsafe { &*ctx.radiant_ctx };
    let sprite_cache = unsafe { &mut *ctx.sprite_cache };
    let game_time = ctx.game_time;

    let mut builder = hecs::EntityBuilder::new();

    // Spatial component (all entities have this)
    builder.add(component::Spatial {
        position: Vec2(px, py),
        angle: Angle(angle),
        lean: 0.0,
    });

    // Hitpoints component (all entities have this)
    builder.add(component::Hitpoints(hitpoints));

    // Bounding component (for collision). Explosions have none, like the original:
    // they are non-interactive, so they can neither take damage nor damage anything
    // (previously overlapping explosions killed each other on the next frame).
    if entity_type != Api::ET_EXPLOSION {
        builder.add(component::Bounding {
            radius: radius,
            faction: faction,
        });
    }

    // Inertial component (most entities have this).
    // Const motion applies v_current * delta directly and never changes it, so the
    // initial velocity passed by the script is the entity's constant drift speed.
    // (The mine AI switches entities to FollowVector via set_v_motion.)
    // Default speed is 100 px/s (asteroids, mines). The player is 5x that (the
    // default felt too slow in playtesting).
    let v_max = if entity_type == Api::ET_PLAYER { 500.0 } else { 100.0 };

    match entity_type {
        Api::ET_EXPLOSION => {
            // Explosions don't move
        }
        _ => {
            builder.add(component::Inertial {
                v_max: Vec2(v_max, v_max),
                v_fraction: Vec2(0.0, 0.0),
                v_current: Vec2(vx, vy),
                trans_motion: 6.0,
                trans_rest: 3.0,
                av_max_v0: 7.0,
                av_max_vmax: 1.4,
                trans_lean: 10.0,
                motion_type: component::InertialMotionType::Const,
            });
        }
    }

    // Lifetime component (if specified)
    // Store absolute expiration time (current age + lifetime) so the cleanup
    // system can compare against ws.age, matching the pattern in def/entity.rs.
    if lifetime > 0.0 {
        builder.add(component::Lifetime(game_time + lifetime));
    }

    // Fading component (if specified): fade alpha 1 -> 0 over the last `fade`
    // seconds of the entity's lifetime (render system applies the fade).
    if fade > 0.0 {
        builder.add(component::Fading {
            start: game_time + lifetime - fade,
            end: game_time + lifetime,
        });
    }

    // Script component (so Itsy can track it)
    builder.add(component::Script(entity_type));

    // Visual component: sprite and layers are referenced by ID (see
    // ScriptContext::sprite_list / Infrastructure layers); the script resolves names to IDs.
    // (Explosions are rendered on the effect layer only — the effects layer is
    // what gets the bloom pass, which is the explosion's glow.)
    let sprite_path = if (sprite_id as usize) < ctx.sprite_list.len() {
        ctx.sprite_list[sprite_id as usize].clone()
    } else {
        eprintln!("spawn_entity: invalid sprite_id {}", sprite_id);
        "res/sprite/placeholder_16x16x1.png".to_string()
    };
    let sprite = match sprite_cache.get(&sprite_path).cloned() {
        Some(s) => s,
        None => {
            match Sprite::from_file(radiant_ctx, &sprite_path) {
                Ok(s) => {
                    let arc = s.arc();
                    sprite_cache.insert(sprite_path.clone(), arc.clone());
                    arc
                }
                Err(e) => {
                    eprintln!("Failed to load sprite '{}': {:?}", sprite_path, e);
                    panic!("Missing sprite: {}", sprite_path);
                }
            }
        }
    };
    // Resolve layers by ID from the Infrastructure (u32::MAX = no layer).
    let layers = unsafe { &(*ctx.infrastructure).layers };
    let resolve_layer = |id: u32| -> Option<Arc<Layer>> {
        if id == Api::LAYER_ID_NONE {
            return None;
        }
        layers.get(id as usize).cloned()
    };
    let layer = resolve_layer(layer_id);
    let effect_layer = resolve_layer(effect_layer_id);

    builder.add(component::Visual {
        layer,
        effect_layer,
        sprite: sprite,
        scale: 1.0,
        effect_scale: 1.0,
        color: Color(color_r, color_g, color_b, 1.0),
        effect_color: Color::WHITE,
        frame_id: 0.0,
        fps: fps,
    });

    cmd.spawn(builder.build());
}
