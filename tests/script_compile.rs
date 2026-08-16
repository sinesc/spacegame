/// Integration test: validate that Itsy scripts compile successfully.
///
/// This test compiles every `.itsy` file under `res/script/` against a minimal
/// API stub using `itsy::build` for detailed error reporting.

use std::path::Path;

/// Minimal context that matches the real `ScriptContext` shape.
#[derive(Clone)]
struct TestContext {}

// Stub API matching the function signatures the game script expects.
itsy::itsy_api! {
    pub Api<TestContext> {
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

/// Recursively collect all `.itsy` files under `dir`.
fn collect_itsy_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    for entry in std::fs::read_dir(dir).expect("read script dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_itsy_files(&path, out);
        } else if path.extension().map_or(false, |e| e == "itsy") {
            out.push(path);
        }
    }
}

#[test]
fn itsy_scripts_compile() {
    let script_dir = Path::new("res/script");
    let mut files = Vec::new();
    collect_itsy_files(script_dir, &mut files);

    if files.is_empty() {
        panic!("no .itsy files found under {}", script_dir.display());
    }

    for file in &files {
        let result = itsy::build::<Api, _>(file);
        match result {
            Ok(_) => {},
            Err(e) => panic!("compile failed for {}:\n{}", file.display(), e),
        }
    }
}
