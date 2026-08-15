use crate::prelude::*;
use hecs;

/// Snapshot of entity state passed to the Itsy script each frame.
#[derive(Clone)]
pub struct EntityData {
    pub hitpoints : f32,
    pub position  : (f32, f32),
    pub velocity  : (f32, f32),
    pub angle     : f32,
    pub script_type: u16,
    pub alive     : bool,
    pub faction   : u32,
}

/// Context shared between Rust and Itsy via the API.
pub struct ScriptContext {
    /// ECS world pointer (set before vm.run(), valid only during that call).
    pub world: *mut hecs::World,
    /// ECS command buffer pointer (set before vm.run(), valid only during that call).
    pub cmd: *mut hecs::CommandBuffer,
    /// Radiant context pointer (for sprite loading in spawn_entity).
    pub radiant_ctx: *const Context,
    /// Sprite cache pointer (for spawn_entity).
    pub sprite_cache: *mut HashMap<String, Arc<Sprite>>,

    /// Snapshot of entity state (rebuilt each frame).
    pub entity_data: HashMap<u64, EntityData>,
    /// Entity ID pairs from the collider (flat: [a, b, c, d, ...] = [(a,b), (c,d)]).
    pub collisions: Vec<u64>,
    /// Entity IDs needing scripted logic this frame.
    pub think_entities: Vec<u64>,
    pub game_time: f32,
    /// Player fire input (set by control system).
    pub input_fire: bool,
    /// Mouse position (set by control system). Note: may be unreliable when cursor is grabbed.  
    /// Use `mouse_delta` for relative movement.
    pub mouse_pos: (f32, f32),
    /// Mouse delta since last frame (set by control system). Reliable even when cursor is grabbed.
    pub mouse_delta: (f32, f32),
    /// Keyboard input flags.
    pub input_keys: u8,  // bit 0=W, 1=S, 2=A, 3=D, 4=R-Shift (strafe)
    /// Spawn trigger from Rust (set each frame, consumed by Itsy).
    pub spawn_trigger: u32,
    /// Entities marked as dying this frame (for on_die dispatch).
    pub dying_entities: Vec<u64>,
    /// Random number generator for the Itsy script (seeded deterministically).
    pub rng: Rng,
}

impl ScriptContext {
    pub fn new() -> Self {
        ScriptContext {
            world: std::ptr::null_mut(),
            cmd: std::ptr::null_mut(),
            radiant_ctx: std::ptr::null(),
            sprite_cache: std::ptr::null_mut(),
            entity_data: HashMap::new(),
            collisions: Vec::new(),
            think_entities: Vec::new(),
            game_time: 0.0,
            input_fire: false,
            mouse_pos: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            input_keys: 0,
            spawn_trigger: 0,
            dying_entities: Vec::new(),
            rng: Rng::new(123.4),
        }
    }
}
