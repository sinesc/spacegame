use crate::prelude::*;
use itsy::internals::binary::heap::HeapRef;

/// Snapshot of entity state passed to the Itsy script each frame.
#[derive(Clone)]
pub struct EntityData {
    pub hitpoints : f32,
    pub position  : (f32, f32),
    pub velocity  : (f32, f32),
    pub angle     : f32,
    pub script_type: u16,
    pub alive     : bool,
}

/// Context shared between Rust and Itsy via the API.
pub struct ScriptContext {
    /// Snapshot of entity state (rebuilt each frame).
    pub entity_data: HashMap<u64, EntityData>,
    /// Entity ID pairs from the collider (flat: [a, b, c, d, ...] = [(a,b), (c,d)]).
    pub collisions: Vec<u64>,
    /// Entity IDs needing scripted logic this frame.
    pub think_entities: Vec<u64>,
    /// Number of commands written by the script this frame.
    pub action_count: u32,
    /// The View<Command>'s heap reference (set by script before suspend).
    pub action_view_ref: HeapRef,
}

impl ScriptContext {
    pub fn new() -> Self {
        ScriptContext {
            entity_data: HashMap::new(),
            collisions: Vec::new(),
            think_entities: Vec::new(),
            action_count: 0,
            action_view_ref: HeapRef::new(0, 0),
        }
    }
}
