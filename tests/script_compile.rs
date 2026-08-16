/// Integration test: validate that the Itsy script compiles successfully.
///
/// This test compiles the entry script `res/script/game.itsy` against a minimal
/// API stub using `itsy::build` for detailed error reporting. Submodules
/// (declared with `mod name;`, e.g. `menu.itsy`) are loaded and checked
/// transitively; only the root module needs a `main` entry function.

use std::path::Path;

/// Minimal context that matches the real `ScriptContext` shape.
#[derive(Clone)]
struct TestContext {}

// Stub API matching the function signatures the game script expects.
itsy::itsy_api! {
    pub Api<TestContext> {
        // Shared protocol constants (must stay in sync with src/scripting/mod.rs).
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
        const ET_NONE         : u16 = 0;
        const ET_PLAYER       : u16 = 1;
        const ET_ASTEROID     : u16 = 2;
        const ET_MINE_RED     : u16 = 3;
        const ET_MINE_GREEN   : u16 = 4;
        const ET_POWERUP_DUAL : u16 = 5;
        const ET_POWERUP_TRIPLE : u16 = 6;
        const ET_PROJECTILE   : u16 = 7;
        const ET_EXPLOSION    : u16 = 8;
        const TRIGGER_NONE      : u32 = 0;
        const TRIGGER_GAME_START: u32 = 4;
        const MOTION_CONST    : u32 = 0;
        const MOTION_FOLLOW   : u32 = 1;
        const MOTION_STRAFE   : u32 = 2;
        const MOTION_DETACHED : u32 = 3;
        const BLEND_NORMAL  : u32 = 0;
        const BLEND_ADD     : u32 = 1;
        const BLEND_LIGHTEN : u32 = 2;
        const FILTER_NONE  : u32 = 0;
        const FILTER_BLOOM : u32 = 1;
        const FILTER_GLARE : u32 = 2;
        const LAYER_ID_NONE : u32 = u32::MAX;

        fn get_hitpoints(&mut _ctx, _id: u64) -> f32 { 0.0 }
        fn get_position_x(&mut _ctx, _id: u64) -> f32 { 0.0 }
        fn get_position_y(&mut _ctx, _id: u64) -> f32 { 0.0 }
        fn get_velocity_x(&mut _ctx, _id: u64) -> f32 { 0.0 }
        fn get_velocity_y(&mut _ctx, _id: u64) -> f32 { 0.0 }
        fn get_angle(&mut _ctx, _id: u64) -> f32 { 0.0 }
        fn get_script_type(&mut _ctx, _id: u64) -> u16 { 0 }
        fn get_faction(&mut _ctx, _id: u64) -> u16 { 0 }
        fn is_alive(&mut _ctx, _id: u64) -> bool { true }
        fn get_think_count(&mut _ctx) -> i32 { 0 }
        fn get_think_id(&mut _ctx, _index: u32) -> u64 { 0 }
        fn get_collision_count(&mut _ctx) -> i32 { 0 }
        fn get_collision_id(&mut _ctx, _index: u32) -> u64 { 0 }
        fn get_game_time(&mut _ctx) -> f32 { 0.0 }
        fn get_mouse_x(&mut _ctx) -> f32 { 0.0 }
        fn get_mouse_y(&mut _ctx) -> f32 { 0.0 }
        fn get_mouse_delta_x(&mut _ctx) -> f32 { 0.0 }
        fn get_mouse_delta_y(&mut _ctx) -> f32 { 0.0 }
        fn get_input_keys(&mut _ctx) -> u16 { 0 }
        fn get_input_pressed(&mut _ctx) -> u16 { 0 }
        fn get_input_edge(&mut _ctx) -> u16 { 0 }
        fn get_spawn_trigger(&mut _ctx) -> u32 { 0 }
        fn get_rand_range(&mut _ctx, _min: f32, _max: f32) -> f32 { 0.0 }
        fn get_sprites(&mut _ctx) -> [ String ] { Vec::new() }
        fn get_sounds(&mut _ctx) -> [ String ] { Vec::new() }
        fn play_sound(&mut _ctx, _id: u32) {}
        fn create_layer(&mut _ctx, _scale: f32, _blendmode: u32) -> u32 { 0 }
        fn add_render_layer(&mut _ctx, _layer_id: u32, _filter: u32, _component: u32) {}
        fn write_text(&mut _ctx, _layer_id: u32, _msg: String, _x: f32, _y: f32, _alpha: f32) {}
        fn set_debug_layer(&mut _ctx, _layer_id: u32) {}
        fn write_menu_text(&mut _ctx, _layer_id: u32, _msg: String, _x: f32, _y: f32, _alpha: f32) {}
        fn pause_time(&mut _ctx) {}
        fn resume_time(&mut _ctx) {}
        fn request_exit(&mut _ctx) {}
        fn request_level_restart(&mut _ctx) {}
        fn toggle_fullscreen(&mut _ctx) {}
        fn get_dying_count(&mut _ctx) -> i32 { 0 }
        fn get_dying_id(&mut _ctx, _index: u32) -> u64 { 0 }
        fn debug_print(&mut _ctx, _msg: String) { }
        fn spawn_entity(&mut _ctx, _entity_type: u16, _sprite_id: u32, _layer_id: u32, _effect_layer_id: u32, _px: f32, _py: f32, _angle: f32, _vx: f32, _vy: f32, _faction: u16, _hitpoints: f32, _radius: f32, _lifetime: f32, _fade: f32, _fps: u32, _color_r: f32, _color_g: f32, _color_b: f32) {}
        fn destroy_entity(&mut _ctx, _entity_id: u64) {}
        fn set_v_motion(&mut _ctx, _entity_id: u64, _motion_type: u32, _vx: f32, _vy: f32) {}
        fn set_angle(&mut _ctx, _entity_id: u64, _angle: f32) {}
        fn set_hitpoints(&mut _ctx, _entity_id: u64, _hp: f32) {}
        fn apply_damage(&mut _ctx, _entity_id: u64, _damage: f32) {}
    }
}

#[test]
fn itsy_scripts_compile() {
    let entry = Path::new("res/script/game.itsy");

    if !entry.is_file() {
        panic!("entry script {} not found", entry.display());
    }

    let result = itsy::build::<Api, _>(entry);
    match result {
        Ok(_) => {},
        Err(e) => panic!("compile failed:\n{}", e),
    }
}
