/// Integration test: validate that Itsy scripts compile successfully.
///
/// This test reads every `.itsy` file under `res/script/` and attempts to
/// compile it against a minimal API stub.  If any script fails to compile
/// the test panics with the compiler error.

use std::fs;
use std::path::Path;

use itsy::internals::binary::heap::HeapRef;

/// Minimal context that matches the real `ScriptContext` shape.
#[derive(Clone)]
struct TestContext {
    _action_count: u32,
    _action_view_ref: HeapRef,
}

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
        fn is_alive(&mut _ctx, _id: u64) -> bool { true }
        fn get_think_count(&mut _ctx) -> i32 { 0 }
        fn get_think_id(&mut _ctx, _index: u32) -> u64 { 0 }
        fn get_collision_count(&mut _ctx) -> i32 { 0 }
        fn get_collision_id(&mut _ctx, _index: u32) -> u64 { 0 }
        fn set_action_count(&mut _ctx, _count: u32) {}
        fn set_action_view_ref(&mut _ctx, heap_ref: HeapRef) {}
    }
}

/// Recursively collect all `.itsy` files under `dir`.
fn collect_itsy_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    for entry in fs::read_dir(dir).expect("read script dir") {
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
        let source = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", file.display(), e));

        let result = itsy::build_str::<Api>(&source);
        assert!(result.is_ok(), "compile failed for {}: {:?}", file.display(), result.err());
    }
}
