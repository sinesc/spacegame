use crate::prelude::*;
use crate::scripting::{Api, ScriptContext, EntityData, ApiOp, SpawnRequest};
use crate::level::component;
use crate::level::Infrastructure;
use crate::level::{RenderLayer, RenderFilter};
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
}

impl Scripting {
    pub fn new(radiant_ctx: Context) -> Self {
        let context = ScriptContext::new();
        Scripting {
            program: None,
            vm: None,
            context,
            radiant_ctx,
            sprite_cache: HashMap::new(),
            sound_cache: HashMap::new(),
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
    pub fn run(&mut self, world: &mut hecs::World, inf: &mut Infrastructure, cmd: &mut hecs::CommandBuffer) {
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
        // `vm.run(&mut self.context)` and the executor's `&mut self` below.
        let mut vm = self.vm.take().unwrap();

        match vm.run(&mut self.context) {
            Ok(itsy::runtime::VMState::Suspended) => {
                // Script suspended — operations recorded in context.pending
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

        // Execute the operations recorded by the Itsy API during vm.run(),
        // in order. World-mutating ops run directly on `world` (before
        // cmd.run_on, as the previous direct API did); spawns go through
        // `cmd` and are applied by the caller (Level::process).
        let pending = std::mem::take(&mut self.context.pending);
        for op in pending {
            self.execute(world, cmd, inf, op);
        }
    }

    /// Execute one API operation recorded during vm.run() (see ApiOp).
    fn execute(&mut self, world: &mut hecs::World, cmd: &mut hecs::CommandBuffer, inf: &mut Infrastructure, op: ApiOp) {
        match op {
            ApiOp::CreateLayer { scale, blend } => {
                let layer = Layer::new((scale * 1920., scale * 1080.)).arc();
                match blend {
                    Api::BLEND_ADD     => { layer.set_blendmode(blendmodes::ADD); }
                    Api::BLEND_LIGHTEN => { layer.set_blendmode(blendmodes::LIGHTEN); }
                    _ => {}
                }
                inf.layers.push(layer);
            }
            ApiOp::AddRenderLayer { layer_id, filter, component } => {
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
            ApiOp::WriteText { layer_id, msg, x, y, alpha, menu } => {
                if let Some(layer) = inf.layers.get(layer_id as usize) {
                    let font = if menu { &inf.menu_font } else { &inf.font };
                    font.write(layer, &msg, (x, y), Color::alpha_pm(alpha));
                }
            }
            ApiOp::SetDebugLayer(layer_id) => {
                inf.debug_layer = layer_id;
            }
            ApiOp::PauseTime => {
                inf.timeframe.lerp_rate(0.0, Duration::from_millis(500));
            }
            ApiOp::ResumeTime => {
                inf.timeframe.lerp_rate(1.0, Duration::from_millis(500));
            }
            ApiOp::RequestExit => {
                inf.exit_requested = true;
            }
            ApiOp::RequestLevelRestart => {
                inf.restart_requested = true;
            }
            ApiOp::ToggleFullscreen => {
                if inf.fullscreen {
                    inf.display.set_windowed();
                    inf.fullscreen = false;
                } else if let Some(monitor) = &inf.monitor {
                    inf.display.set_fullscreen(Some(monitor.clone())).unwrap();
                    inf.fullscreen = true;
                }
            }
            ApiOp::PlaySound { id } => {
                let name = self.context.sound_list[id as usize].clone();
                if self.sound_cache.get(&name).is_none() {
                    match Sound::load(&name) {
                        Ok(sound) => { self.sound_cache.insert(name.clone(), sound); }
                        Err(e) => { eprintln!("play_sound: failed to load '{}': {}", name, e); return; }
                    }
                }
                let sound = self.sound_cache.get(&name).unwrap();
                inf.audio.add(sound.decoder());
            }
            ApiOp::Spawn(req) => {
                self.spawn_entity(req, cmd, inf);
            }
            ApiOp::Despawn(entity_id) => {
                if let Some(entity) = hecs::Entity::from_bits(entity_id) {
                    if let Err(e) = world.despawn(entity) {
                        eprintln!("destroy_entity(id={}) failed: {:?}", entity_id, e);
                    }
                }
            }
            ApiOp::SetVMotion { id, motion, vx, vy } => {
                if let Some(entity) = hecs::Entity::from_bits(id) {
                    if let Ok(mut inertial) = world.get::<&mut component::Inertial>(entity) {
                        inertial.v_fraction = Vec2(vx, vy);
                        inertial.motion_type = match motion {
                            Api::MOTION_CONST  => component::InertialMotionType::Const,
                            Api::MOTION_FOLLOW => component::InertialMotionType::FollowVector,
                            Api::MOTION_STRAFE => component::InertialMotionType::StrafeVector,
                            _ => component::InertialMotionType::Detached,
                        };
                    }
                }
            }
            ApiOp::SetAngle { id, angle } => {
                if let Some(entity) = hecs::Entity::from_bits(id) {
                    if let Ok(mut spatial) = world.get::<&mut component::Spatial>(entity) {
                        spatial.angle = Angle(angle);
                    }
                }
            }
            ApiOp::SetHitpoints { id, hp } => {
                if let Some(entity) = hecs::Entity::from_bits(id) {
                    if let Ok(mut hitpoints) = world.get::<&mut component::Hitpoints>(entity) {
                        hitpoints.0 = hp;
                    }
                }
            }
            ApiOp::ApplyDamage { id, damage } => {
                if let Some(entity) = hecs::Entity::from_bits(id) {
                    if let Ok(mut hitpoints) = world.get::<&mut component::Hitpoints>(entity) {
                        hitpoints.0 -= damage;
                    }
                }
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

    /// Create an ECS entity from a queued spawn request.
    fn spawn_entity(&mut self, req: SpawnRequest, cmd: &mut hecs::CommandBuffer, inf: &Infrastructure) {

        let SpawnRequest {
            entity_type, sprite_id, layer_id, effect_layer_id,
            px, py, angle, vx, vy, faction,
            hitpoints, radius, lifetime, fade, fps,
            color_r, color_g, color_b, game_time,
        } = req;

        let mut builder = hecs::EntityBuilder::new();

        let layers = &inf.layers;

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
        let v_max = if entity_type == Api::ET_PLAYER { 750.0 } else { 100.0 };

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
        let sprite_path = if (sprite_id as usize) < self.context.sprite_list.len() {
            self.context.sprite_list[sprite_id as usize].clone()
        } else {
            eprintln!("spawn_entity: invalid sprite_id {}", sprite_id);
            "res/sprite/placeholder_16x16x1.png".to_string()
        };
        let sprite = match self.sprite_cache.get(&sprite_path).cloned() {
            Some(s) => s,
            None => {
                match Sprite::from_file(&self.radiant_ctx, &sprite_path) {
                    Ok(s) => {
                        let arc = s.arc();
                        self.sprite_cache.insert(sprite_path.clone(), arc.clone());
                        arc
                    }
                    Err(e) => {
                        eprintln!("Failed to load sprite '{}': {:?}", sprite_path, e);
                        panic!("Missing sprite: {}", sprite_path);
                    }
                }
            }
        };
        // Resolve layers by ID (u32::MAX = no layer).
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
}
