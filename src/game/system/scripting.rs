use crate::prelude::*;
use crate::scripting::{Api, ScriptContext, EntityData, ApiOp, SpawnRequest};
use crate::game::component;
use crate::game::{Infrastructure, State};
use crate::game::system::render::{RenderLayer, RenderFilter, RenderBackground};
use hecs;
use itsy;

/// The scripting subsystem: owns the Itsy VM and manages the frame cycle.
pub struct Scripting {
    program : Option<itsy::Program<Api>>,
    /// Persistent VM so that suspend/resume works and local state survives across frames.
    vm: Option<itsy::runtime::VM<Api, ScriptContext>>,
    context: ScriptContext,
}

impl Scripting {
    pub fn new() -> Self {
        Scripting {
            program: None,
            vm: None,
            context: ScriptContext::new(),
        }
    }

    /// Prepare scripting state/input prior to script processing.
    pub fn prepare_frame(self: &mut Self, world: &mut hecs::World, inf: &mut Infrastructure, state: &mut State, age: f32) {

        self.prepare_collision_pairs(world);
        self.prepare_keys(&inf.input);

        self.context.game_time = age;
        self.context.mouse_pos = inf.input.mouse();
        self.context.mouse_delta = inf.input.mouse_delta();
        self.context.screen_size = inf.display.dimensions();

        // Send GAME_START trigger on first frame
        if !state.game_started {
            self.context.spawn_trigger = Api::TRIGGER_GAME_START;
            state.game_started = true;
        }
    }

    /// Run the Itsy script for one frame.
    pub fn run(&mut self, world: &mut hecs::World, inf: &mut Infrastructure, state: &mut State, cmd: &mut hecs::CommandBuffer) {
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
        // Background draws are rebuilt each frame (the script re-requests
        // them), so clear the previous frame's list first.
        inf.background_draws.clear();
        let pending = std::mem::take(&mut self.context.pending);
        for op in pending {
            self.execute_command(world, cmd, inf, state, op);
        }
    }

    /// Execute one API operation recorded during vm.run() (see ApiOp).
    fn execute_command(&mut self, world: &mut hecs::World, cmd: &mut hecs::CommandBuffer, inf: &mut Infrastructure, state: &mut State, op: ApiOp) {
        match op {
            ApiOp::CreateLayer { scale, blend } => {
                let (w, h) = inf.display.dimensions();
                let layer = Layer::new((scale * w as f32, scale * h as f32)).arc();
                match blend {
                    Api::BLEND_ADD     => { layer.set_blendmode(blendmodes::ADD); }
                    Api::BLEND_LIGHTEN => { layer.set_blendmode(blendmodes::LIGHTEN); }
                    _ => {}
                }
                inf.layers.push(layer);
                inf.layer_scales.push(scale);
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
                state.timeframe.lerp_rate(0.0, Duration::from_millis(500));
            }
            ApiOp::ResumeTime => {
                state.timeframe.lerp_rate(1.0, Duration::from_millis(500));
            }
            ApiOp::RequestExit => {
                state.exit_requested = true;
            }
            ApiOp::RequestLevelRestart => {
                state.restart_requested = true;
            }
            ApiOp::SetResolution { width, height } => {
                eprintln!("[debug] SetResolution op: ({width}, {height})");
                state.resolution_requested = Some((width, height));
            }
            ApiOp::ToggleFullscreen => {
                eprintln!("[debug] ToggleFullscreen op, state.fullscreen = {}", state.fullscreen);
                if state.fullscreen {
                    // Capture the fullscreen size before switching, so the shrink
                    // target is 3/4 of the monitor size.
                    let (w, h) = inf.display.dimensions();
                    inf.display.set_windowed();
                    state.fullscreen = false;
                    // The window keeps the monitor size after leaving fullscreen
                    // and would look identical to fullscreen; shrink it so the
                    // change is visible. Deferred via resolution_requested:
                    // set_dimensions must not run mid-frame (it presents the
                    // in-flight frame, leaving the target unprepared for the
                    // rest of this frame's draws) — the main loop applies it
                    // after swap_frame.
                    let (sw, sh) = ((w * 3) / 4, (h * 3) / 4);
                    eprintln!("[debug] ToggleFullscreen: -> windowed, deferring shrink to ({sw}, {sh})");
                    state.resolution_requested = Some((sw, sh));
                } else {
                    // None = primary monitor. The monitor captured at startup can be
                    // None (Display::monitors() is empty before the first event pump),
                    // so don't depend on it.
                    match inf.display.set_fullscreen(None) {
                        Ok(()) => { state.fullscreen = true; }
                        Err(e) => { eprintln!("toggle_fullscreen: failed to enter fullscreen: {:?}", e); }
                    }
                    eprintln!("[debug] ToggleFullscreen: -> fullscreen");
                }
            }
            ApiOp::PlaySound { id } => {
                let name = self.context.sound_list[id as usize].clone();
                if inf.sound_cache.get(&name).is_none() {
                    match crate::sound::Sound::load(&name) {
                        Ok(sound) => { inf.sound_cache.insert(name.clone(), sound); }
                        Err(e) => { eprintln!("play_sound: failed to load '{}': {}", name, e); return; }
                    }
                }
                let sound = inf.sound_cache.get(&name).unwrap();
                inf.audio.add(sound.decoder());
            }
            ApiOp::DrawBackground { id, offset_x, offset_y } => {
                let name = self.context.background_list[id as usize].clone();
                let texture = match inf.background_cache.get(&name).cloned() {
                    Some(t) => t,
                    None => {
                        match Texture::from_file(&inf.radiant_ctx, &name) {
                            Ok(t) => {
                                let arc = Arc::new(t);
                                inf.background_cache.insert(name.clone(), arc.clone());
                                arc
                            }
                            Err(e) => { eprintln!("draw_background: failed to load '{}': {:?}", name, e); return; }
                        }
                    }
                };
                inf.background_draws.push(RenderBackground { texture, offset_x, offset_y });
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
    fn spawn_entity(&mut self, req: SpawnRequest, cmd: &mut hecs::CommandBuffer, inf: &mut Infrastructure) {

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
        let sprite = match inf.sprite_cache.get(&sprite_path).cloned() {
            Some(s) => s,
            None => {
                match Sprite::from_file(&inf.radiant_ctx, &sprite_path) {
                    Ok(s) => {
                        let arc = s.arc();
                        inf.sprite_cache.insert(sprite_path.clone(), arc.clone());
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

    /// Identifies pressed keys.
    fn prepare_keys(self: &mut Self, input: &Input) {

        // Input masks: one bit per key (see KEY_* in scripting/mod.rs).
        // `keys` = held down, `pressed` = pressed this frame (incl. repeats),
        // `edge` = initial press this frame (no repeats).
        let (keys, pressed, edge) = {
            let mut keys = 0u16;
            let mut pressed = 0u16;
            let mut edge = 0u16;
            let mut add_key = |key: InputId, bit: u16| {
                if input.down(key) { keys |= bit; }
                if input.pressed(key, true) { pressed |= bit; }
                if input.pressed(key, false) { edge |= bit; }
            };
            add_key(InputId::W, Api::KEY_W);
            add_key(InputId::S, Api::KEY_S);
            add_key(InputId::A, Api::KEY_A);
            add_key(InputId::D, Api::KEY_D);
            add_key(InputId::RShift, Api::KEY_RSHIFT);
            add_key(InputId::CursorUp, Api::KEY_CURSOR_UP);
            add_key(InputId::CursorDown, Api::KEY_CURSOR_DOWN);
            add_key(InputId::Return, Api::KEY_RETURN);
            add_key(InputId::Escape, Api::KEY_ESCAPE);
            add_key(InputId::Mouse1, Api::KEY_MOUSE1);
            add_key(InputId::LControl, Api::KEY_LCONTROL);
            (keys, pressed, edge)
        };

        self.context.input_keys = keys;
        self.context.input_pressed = pressed;
        self.context.input_edge = edge;
    }

    /// Detect collision pairs for the scripting subsystem.
    /// Returns flat list: [a, b, c, d, ...] = [(a,b), (c,d)].
    fn prepare_collision_pairs(self: &mut Self, world: &hecs::World) {
        let entities: Vec<(hecs::Entity, Vec2, f32, u16)> = world
            .query::<(&component::Spatial, &component::Bounding, &component::Hitpoints)>()
            .iter()
            .map(|(e, (s, b, _))| (e, s.position, b.radius, b.faction))
            .collect();

        let mut pairs = Vec::new();
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let &(ea, pos_a, rad_a, fac_a) = &entities[i];
                let &(eb, pos_b, rad_b, fac_b) = &entities[j];
                if fac_a != fac_b && rad_a + rad_b > pos_a.distance(&pos_b) {
                    pairs.push(ea.to_bits().into());
                    pairs.push(eb.to_bits().into());
                }
            }
        }
        self.context.collisions = pairs;
    }
}
