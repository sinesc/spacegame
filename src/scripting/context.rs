use crate::prelude::*;

/// Snapshot of entity state passed to the Itsy script each frame.
#[derive(Clone)]
pub struct EntityData {
    pub hitpoints : f32,
    pub position  : (f32, f32),
    pub velocity  : (f32, f32),
    pub angle     : f32,
    pub script_type: u16,
    pub alive     : bool,
    pub faction   : u16,
}

/// Parameters for a queued entity spawn (`ApiOp::Spawn`).
pub struct SpawnRequest {
    pub entity_type : u16,
    pub sprite_id   : u32,
    pub layer_id    : u32,
    pub effect_layer_id : u32,
    pub px          : f32,
    pub py          : f32,
    pub angle       : f32,
    pub vx          : f32,
    pub vy          : f32,
    pub faction     : u16,
    pub hitpoints   : f32,
    pub radius      : f32,
    pub lifetime    : f32,
    pub fade        : f32,
    pub fps         : u32,
    pub color_r     : f32,
    pub color_g     : f32,
    pub color_b     : f32,
    /// Game time at queue time (used to resolve Lifetime/Fading deadlines).
    pub game_time   : f32,
}

/// Operations recorded by the Itsy API during vm.run(). The Scripting system
/// executes them (in order) after vm.run() returns, with plain borrows of the
/// ECS world, command buffer, caches and Infrastructure. This keeps the API
/// functions free of raw pointers.
pub enum ApiOp {
    CreateLayer { scale: f32, blend: u32 },
    AddRenderLayer { layer_id: u32, filter: u32, component: u32 },
    WriteText { layer_id: u32, msg: String, x: f32, y: f32, alpha: f32, menu: bool },
    SetDebugLayer(u32),
    /// Draw a background image (index into `background_list`) with a scroll
    /// offset in screen pixels; the image wraps for infinite scrolling.
    DrawBackground { id: u32, offset_x: f32, offset_y: f32 },
    PauseTime,
    ResumeTime,
    ToggleFullscreen,
    /// Resize the display (applied by the main loop after swap_frame).
    SetResolution { width: u32, height: u32 },
    RequestExit,
    RequestLevelRestart,
    PlaySound { id: u32 },
    Spawn(SpawnRequest),
    Despawn(u64),
    SetVMotion { id: u64, motion: u32, vx: f32, vy: f32 },
    SetAngle { id: u64, angle: f32 },
    SetHitpoints { id: u64, hp: f32 },
    ApplyDamage { id: u64, damage: f32 },
}

/// Context shared between Rust and Itsy via the API.
pub struct ScriptContext {
    /// Operations recorded by API calls during the current vm.run(); drained
    /// and executed by the Scripting system after vm.run() returns.
    pub pending: Vec<ApiOp>,
    /// ID (Infrastructure::layers index) returned by the next create_layer
    /// call. FIFO execution of CreateLayer ops keeps this in sync with the
    /// actual layer vector (only CreateLayer appends to it).
    pub next_layer_id: u32,

    /// Snapshot of entity state (rebuilt each frame).
    pub entity_data: HashMap<u64, EntityData>,
    /// Entity ID pairs from the collider (flat: [a, b, c, d, ...] = [(a,b), (c,d)]).
    pub collisions: Vec<u64>,
    /// Entity IDs needing scripted logic this frame.
    pub think_entities: Vec<u64>,
    pub game_time: f32,
    /// Mouse position (set by control system). Note: may be unreliable when cursor is grabbed.
    /// Use `mouse_delta` for relative movement.
    pub mouse_pos: (i32, i32),
    /// Mouse delta since last frame (set by control system). Reliable even when cursor is grabbed.
    pub mouse_delta: (i32, i32),
    /// Current display size in pixels (set by the control system each frame).
    pub screen_size: (u32, u32),
    /// Keyboard input masks (see KEY_* constants in scripting/mod.rs).
    /// `input_keys` = keys currently held down.
    pub input_keys: u16,
    /// Keys pressed this frame, including repeat events while held.
    pub input_pressed: u16,
    /// Keys pressed this frame, initial press only (no repeats).
    pub input_edge: u16,
    /// Spawn trigger from Rust (set each frame, consumed by Itsy).
    pub spawn_trigger: u32,
    /// Entities marked as dying this frame (for on_die dispatch).
    pub dying_entities: Vec<u64>,
    /// Random number generator for the Itsy script (seeded deterministically).
    pub rng: Rng,
    /// Sprite file paths (recursive listing of res/sprite, sorted).
    /// Generated once at startup; the vector index is the sprite ID
    /// shared between Itsy and Rust.
    pub sprite_list: Vec<String>,
    /// Sound file paths (recursive listing of res/sound, sorted).
    /// Generated once at startup; the vector index is the sound ID
    /// shared between Itsy and Rust.
    pub sound_list: Vec<String>,
    /// Background image file paths (recursive listing of res/background,
    /// sorted). Generated once at startup; the vector index is the background
    /// ID shared between Itsy and Rust.
    pub background_list: Vec<String>,
}

impl ScriptContext {
    pub fn new() -> Self {
        ScriptContext {
            pending: Vec::new(),
            next_layer_id: 0,
            entity_data: HashMap::new(),
            collisions: Vec::new(),
            think_entities: Vec::new(),
            game_time: 0.0,
            mouse_pos: (0, 0),
            mouse_delta: (0, 0),
            screen_size: (0, 0),
            input_keys: 0,
            input_pressed: 0,
            input_edge: 0,
            spawn_trigger: 0,
            dying_entities: Vec::new(),
            rng: Rng::new(123.4),
            sprite_list: list_files_recursive("res/sprite"),
            sound_list: list_files_recursive("res/sound"),
            background_list: list_files_recursive("res/background"),
        }
    }
}

/// Recursively lists all files under `path`, returning project-root-relative
/// paths in a stable (sorted) order so vector indices stay deterministic.
fn list_files_recursive(path: &str) -> Vec<String> {
    let mut result = Vec::new();
    list_files_recursive_inner(path, &mut result);
    result.sort();
    result
}

fn list_files_recursive_inner(path: &str, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(path) else {
        eprintln!("list_files_recursive: cannot read directory '{}'", path);
        return;
    };
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() {
            subdirs.push(format!("{}/{}", path, name));
        } else {
            out.push(format!("{}/{}", path, name));
        }
    }
    subdirs.sort();
    for sub in subdirs {
        list_files_recursive_inner(&sub, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_sprite_files() {
        let sprites = list_files_recursive("res/sprite");
        assert!(sprites.len() > 0, "expected sprite files");
        assert!(sprites.iter().all(|p| p.starts_with("res/sprite/")));
        // stable ordering: sorted
        let mut sorted = sprites.clone();
        sorted.sort();
        assert_eq!(sprites, sorted);
    }

    #[test]
    fn lists_background_files() {
        let backgrounds = list_files_recursive("res/background");
        assert!(backgrounds.len() > 0, "expected background files");
        assert!(backgrounds.iter().all(|p| p.starts_with("res/background/")));
        // stable ordering: sorted
        let mut sorted = backgrounds.clone();
        sorted.sort();
        assert_eq!(backgrounds, sorted);
        assert!(backgrounds.contains(&"res/background/blue.jpg".to_string()));
    }
}