mod context;

pub use self::context::{ScriptContext, EntityData};

use crate::prelude::*;
use crate::level::component;
use crate::level::WorldState;
use hecs;
use itsy;
use itsy::internals::binary::heap::HeapRef;
use std::fs;

/// The scripting subsystem: owns the Itsy VM and manages the frame cycle.
pub struct ScriptingSubsystem {
    program : Option<itsy::Program<Api>>,
    context : ScriptContext,
}

// Define the Itsy API type.
itsy::itsy_api! {
    pub Api<ScriptContext> {
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
        fn set_action_count(&mut context, count: u32) {
            context.action_count = count;
        }
        fn set_action_view_ref(&mut context, ref_: HeapRef) {
            context.action_view_ref = ref_;
        }
    }
}

impl ScriptingSubsystem {
    pub fn new() -> Self {
        ScriptingSubsystem {
            program: None,
            context: ScriptContext::new(),
        }
    }

    /// Run the Itsy script for one frame.
    pub fn run(&mut self, world: &mut hecs::World, _ws: &WorldState, cmd: &mut hecs::CommandBuffer) {
        // Build entity snapshot
        self.build_snapshot(world);

        // Load and compile the script once
        if self.program.is_none() {
            let source = match fs::read_to_string("res/script/game.itsy") {
                Ok(s) => s,
                Err(_) => return, // no script file, skip
            };

            match itsy::build_str::<Api>(&source) {
                Ok(p) => { self.program = Some(p); }
                Err(e) => {
                    eprintln!("Script compile error: {:?}", e);
                    return;
                }
            }
        }

        let program = self.program.take().unwrap();
        let mut vm = itsy::runtime::VM::new(program);
        match vm.run(&mut self.context) {
            Ok(itsy::runtime::VMState::Suspended) => {
                // Script suspended — read action buffer
                self.apply_actions(&vm, world, cmd);
            }
            Ok(itsy::runtime::VMState::Terminated) | Ok(itsy::runtime::VMState::Ready) => {
                // Script finished without suspending; nothing to do
            }
            Ok(itsy::runtime::VMState::Error(_)) | Err(_) => {
                eprintln!("Script error");
            }
        }
    }

    /// Set collision pairs from the collider system.
    /// Pairs are passed as flat list: [a, b, c, d, ...] = [(a,b), (c,d)].
    pub fn set_collisions(&mut self, pairs: Vec<u64>) {
        self.context.collisions = pairs;
    }

    /// Build the entity data snapshot from the ECS world.
    fn build_snapshot(&mut self, world: &hecs::World) {
        self.context.entity_data.clear();
        self.context.think_entities.clear();
        self.context.collisions.clear();

        // Query all entities with Script component
        world
            .query::<(&component::Script, &component::Spatial, &component::Hitpoints, Option<&component::Inertial>)>()
            .iter()
            .for_each(|(e, (script, spatial, hp, inertial))| {
                let vel = inertial.map(|i| (i.v_current.0, i.v_current.1)).unwrap_or((0.0, 0.0));
                self.context.entity_data.insert(
                    e.to_bits().into(),
                    EntityData {
                        hitpoints: hp.0,
                        position: (spatial.position.0, spatial.position.1),
                        velocity: vel,
                        angle: spatial.angle.0,
                        script_type: script.0,
                        alive: true,
                    },
                );
                self.context.think_entities.push(e.to_bits().into());
            });
    }

    /// Read the action buffer from the Itsy heap and apply commands.
    fn apply_actions(&self, vm: &itsy::runtime::VM<Api, ScriptContext>, world: &mut hecs::World, _cmd: &mut hecs::CommandBuffer) {
        let ref_ = self.context.action_view_ref;
        let count = self.context.action_count as usize;

        if count == 0 {
            return;
        }

        let heap = &vm.heap;
        let view_index = ref_.index();
        let view_obj = heap.item(view_index);
        let data = &view_obj.data;

        // Command enum layout:
        // Each slot = discriminant (2 bytes, u16 LE) + max variant payload (20 bytes)
        // Total slot size = 22 bytes
        //
        // Variant discriminants (0-indexed):
        //   0 = SpawnEntity(entity_def_index: u32, position_x: f32, position_y: f32) — 12 bytes
        //   1 = DestroyEntity(entity_id: u64) — 8 bytes
        //   2 = SetVelocity(entity_id: u64, vx: f32, vy: f32) — 16 bytes
        //   3 = SetAngle(entity_id: u64, angle: f32) — 12 bytes
        //   4 = SetHitpoints(entity_id: u64, hp: f32) — 12 bytes
        //   5 = ApplyDamage(entity_id: u64, damage: f32) — 12 bytes
        //
        // Payload fields are little-endian: u32=4B, f32=4B, u64=8B

        const SLOT_SIZE: usize = 22; // 2 (disc) + 20 (max payload)

        for i in 0..count {
            let base = i * SLOT_SIZE;
            if base + SLOT_SIZE > data.len() {
                eprintln!("Command buffer overrun at index {}", i);
                break;
            }

            // Read discriminant (u16 LE)
            let disc = u16::from_le_bytes([data[base], data[base + 1]]);
            let payload = &data[base + 2..base + SLOT_SIZE];

            match disc {
                0 => {
                    // SpawnEntity(u32, f32, f32)
                    let entity_def_index = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let px = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
                    let py = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
                    eprintln!("CMD SpawnEntity(index={}, pos=({}, {}))", entity_def_index, px, py);
                    // TODO: requires entity repository access
                }
                1 => {
                    // DestroyEntity(u64)
                    let entity_id = u64::from_le_bytes([
                        payload[0], payload[1], payload[2], payload[3],
                        payload[4], payload[5], payload[6], payload[7],
                    ]);
                    if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                        if let Err(e) = world.despawn(entity) {
                            eprintln!("CMD DestroyEntity(id={}) failed: {:?}", entity_id, e);
                        }
                    }
                }
                2 => {
                    // SetVelocity(u64, f32, f32)
                    let entity_id = u64::from_le_bytes([
                        payload[0], payload[1], payload[2], payload[3],
                        payload[4], payload[5], payload[6], payload[7],
                    ]);
                    let vx = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
                    let vy = f32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);
                    if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                        if let Ok(mut inertial) = world.get::<&mut component::Inertial>(entity) {
                            inertial.v_current = Vec2(vx, vy);
                        }
                    }
                }
                3 => {
                    // SetAngle(u64, f32)
                    let entity_id = u64::from_le_bytes([
                        payload[0], payload[1], payload[2], payload[3],
                        payload[4], payload[5], payload[6], payload[7],
                    ]);
                    let angle = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
                    if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                        if let Ok(mut spatial) = world.get::<&mut component::Spatial>(entity) {
                            spatial.angle = Angle(angle);
                        }
                    }
                }
                4 => {
                    // SetHitpoints(u64, f32)
                    let entity_id = u64::from_le_bytes([
                        payload[0], payload[1], payload[2], payload[3],
                        payload[4], payload[5], payload[6], payload[7],
                    ]);
                    let hp = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
                    if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                        if let Ok(mut hitpoints) = world.get::<&mut component::Hitpoints>(entity) {
                            hitpoints.0 = hp;
                        }
                    }
                }
                5 => {
                    // ApplyDamage(u64, f32)
                    let entity_id = u64::from_le_bytes([
                        payload[0], payload[1], payload[2], payload[3],
                        payload[4], payload[5], payload[6], payload[7],
                    ]);
                    let damage = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
                    if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                        if let Ok(mut hitpoints) = world.get::<&mut component::Hitpoints>(entity) {
                            hitpoints.0 -= damage;
                        }
                    }
                }
                _ => {
                    eprintln!("Unknown command discriminant: {}", disc);
                }
            }
        }
    }
}
