use crate::prelude::*;
use crate::scripting::{Api, ScriptContext, EntityData};
use crate::level::component;
use crate::level::Infrastructure;
use crate::level::WorldState;
use crate::sound::Sound;
use hecs;
use itsy;
use std::collections::HashMap;

/// The scripting subsystem: owns the Itsy VM and manages the frame cycle.
pub struct Scripting {
    program : Option<itsy::Program<Api>>,
    /// Persistent VM so that suspend/resume works and local state survives across frames.
    vm: Option<itsy::runtime::VM<Api, ScriptContext>>,
    context: ScriptContext,
    /// Radiant context for loading sprites
    radiant_ctx: Context,
    /// Sprite cache
    sprite_cache: HashMap<String, Arc<Sprite>>,
    /// Sound cache (loaded on first play)
    sound_cache: HashMap<String, Sound>,
    /// Keep the Infrastructure alive for the lifetime of the subsystem
    /// (`context.infrastructure` points into it).
    _inf: Arc<Infrastructure>,
}

impl Scripting {
    /// `inf` references the Infrastructure, which outlives the scripting
    /// subsystem.
    pub fn new(radiant_ctx: Context, inf: Arc<Infrastructure>) -> Self {
        let mut context = ScriptContext::new();
        context.infrastructure = Arc::as_ptr(&inf) as *mut Infrastructure;
        Scripting {
            program: None,
            vm: None,
            context,
            radiant_ctx,
            sprite_cache: HashMap::new(),
            sound_cache: HashMap::new(),
            _inf: inf,
        }
    }

    /// Set the game time for Itsy timers.
    pub fn set_game_time(&mut self, age: f32) {
        self.context.game_time = age;
    }

    /// Set mouse position.
    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.context.mouse_pos = (x, y);
    }

    /// Set mouse delta (movement since last frame).
    pub fn set_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.context.mouse_delta = (dx, dy);
    }

    /// Set the keyboard input masks for this frame (see KEY_* constants).
    /// `keys` = currently held down, `pressed` = pressed this frame (incl. repeats),
    /// `edge` = initial press this frame (no repeats).
    pub fn set_input_state(&mut self, keys: u16, pressed: u16, edge: u16) {
        self.context.input_keys = keys;
        self.context.input_pressed = pressed;
        self.context.input_edge = edge;
    }

    /// Set a spawn trigger for Itsy to process.
    pub fn set_spawn_trigger(&mut self, trigger: u32) {
        self.context.spawn_trigger = trigger;
    }

    /// Run the Itsy script for one frame.
    pub fn run(&mut self, world: &mut hecs::World, _ws: &WorldState, cmd: &mut hecs::CommandBuffer) {
        // Build entity snapshot
        self.build_snapshot(world);

        // Load and compile the script once (itsy::build also resolves `mod` declarations,
        // e.g. `mod menu;` -> res/script/menu.itsy, relative to the source file).
        if self.program.is_none() {
            match itsy::build::<Api, _>("res/script/game.itsy") {
                Ok(p) => { self.program = Some(p); }
                Err(e) => {
                    eprintln!("Script compile error: {}", e);
                    return;
                }
            }
        }

        // Create the persistent VM once (first frame).  On subsequent frames we
        // resume from the `suspend` instruction, preserving all local state.
        if self.vm.is_none() {
            if let Some(program) = self.program.take() {
                self.vm = Some(itsy::runtime::VM::new(program));
            } else {
                return;
            }
        }

        // Take the VM out of self to avoid borrow conflicts between
        // `vm.run(&mut self.context)` and `self.apply_actions(...)`.
        let mut vm = self.vm.take().unwrap();

        // Set world/cmd pointers so API functions can access them directly.
        // Valid only during vm.run().
        self.context.world = world as *mut hecs::World;
        self.context.cmd = cmd as *mut hecs::CommandBuffer;
        self.context.radiant_ctx = &self.radiant_ctx;
        self.context.sprite_cache = &mut self.sprite_cache;
        self.context.sound_cache = &mut self.sound_cache;

        match vm.run(&mut self.context) {
            Ok(itsy::runtime::VMState::Suspended) => {
                // Script suspended — commands executed directly via API
            }
            Ok(itsy::runtime::VMState::Terminated) | Ok(itsy::runtime::VMState::Ready) => {
                // Script finished without suspending (shouldn't happen with while-loop).
                // Reset and let it restart on the next frame.
                eprintln!("Script terminated unexpectedly, resetting VM");
                vm.reset();
            }
            Ok(itsy::runtime::VMState::Error(_)) => {
                eprintln!("Script error, resetting VM");
                vm.reset();
            }
            Err(e) => {
                eprintln!("Script error: {:?}, resetting VM", e);
                vm.reset();
            }
        }

        // Put the VM back so it persists for the next frame.
        self.vm = Some(vm);
    }

    /// Set collision pairs from the collider system.
    /// Pairs are passed as flat list: [a, b, c, d, ...] = [(a,b), (c,d)].
    pub fn set_collisions(&mut self, pairs: Vec<u64>) {
        self.context.collisions = pairs;
    }

    /// Build the entity data snapshot from the ECS world.
    fn build_snapshot(&mut self, world: &hecs::World) {
        // Save old entity data to detect deaths (entities that were alive last
        // frame but are no longer in the world).
        let old_data = std::mem::take(&mut self.context.entity_data);

        self.context.think_entities.clear();
        self.context.collisions.clear();
        self.context.dying_entities.clear();

        // Query all entities with Script component
        let mut new_ids = std::collections::HashSet::new();
        world
            .query::<(&component::Script, &component::Spatial, &component::Hitpoints, Option<&component::Inertial>, Option<&component::Bounding>)>()
            .iter()
            .for_each(|(e, (script, spatial, hp, inertial, bounding))| {
                let id = e.to_bits().into();
                new_ids.insert(id);
                let vel = inertial.map(|i| (i.v_current.0, i.v_current.1)).unwrap_or((0.0, 0.0));
                let faction = bounding.map(|b| b.faction).unwrap_or(0);
                self.context.entity_data.insert(
                    id,
                    EntityData {
                        hitpoints: hp.0,
                        position: (spatial.position.0, spatial.position.1),
                        velocity: vel,
                        angle: spatial.angle.0,
                        script_type: script.0,
                        alive: true,
                        faction: faction,
                    },
                );
                self.context.think_entities.push(id);
            });

        // Detect dead entities: were in the snapshot last frame but are gone now.
        // Keep their data in entity_data (with alive=false) so the Itsy script can
        // still look up their type/faction via get_script_type/get_faction, and
        // dispatch on_die handlers.
        // Only report entities that were alive last frame — entities already marked
        // dead in a previous frame should not be re-dispatched.
        for (id, mut data) in old_data {
            if !new_ids.contains(&id) {
                if data.alive {
                    self.context.dying_entities.push(id);
                }
                data.alive = false;
                self.context.entity_data.insert(id, data);
            }
        }
    }
}
