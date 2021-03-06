use serde::de::{self, Deserializer, Deserialize};
use specs;
use prelude::*;
use def::{parse_dir, Error};
use level::component::*;
use repository::Repository;
use def::spawner::*;
use def::faction::*;
use serde_yaml;

// set up some ugly unsafe global state to work around missing DeserializeSeed in Serde-Yaml
static mut FACTIONS: *const Vec<String> = 0 as _;
static mut SPRITES: *mut Repository<Arc<Sprite>> = 0 as _;
static mut LAYERS: *const Repository<Arc<Layer>> = 0 as _;
static mut SPAWNERS: *const Repository<SpawnerDescriptor, SpawnerId> = 0 as _;
static mut CONTEXT: *const RenderContext = 0 as _;

pub fn parse_entities(context: &RenderContext, sprites: &mut Repository<Arc<Sprite>>, factions: &Vec<String>, spawners: &mut Repository<SpawnerDescriptor, SpawnerId>, layers: &Repository<Arc<Layer>>) -> Result<Repository<EntityDescriptor>, Error> {
    unsafe {
        FACTIONS = factions as *const Vec<String>;
        SPRITES = sprites as *mut Repository<Arc<Sprite>>;
        LAYERS = layers as *const Repository<Arc<Layer>>;
        SPAWNERS = spawners as *const Repository<SpawnerDescriptor, SpawnerId>;
        CONTEXT = context as *const RenderContext;
    }

    // parse entities to values first. spawners allow derival of custom entities from base entities.
    // the derival is implemented as a merge operation between the base entity yaml map and the derived entity yaml map

    let entity_values: HashMap<String, serde_yaml::Value> = parse_dir("res/def/entity/", &[ "yaml" ])?;
    complete_spawners(spawners, &entity_values);

    // finally deserialize entities into a repository of entities
    Ok(entity_values.iter().map(|(k, v)| (k.clone(), serde_yaml::from_value(v.clone()).unwrap())).collect())
}

#[derive(Deserialize, Debug)]
pub struct EntityDescriptor {
    bounding    : Option<Bounding>,
    computed    : Option<Computed>,
    controlled  : Option<Controlled>,
    explodes    : Option<Explodes>,
    fading      : Option<Fading>,
    hitpoints   : Option<Hitpoints>,
    inertial    : Option<Inertial>,
    lifetime    : Option<Lifetime>,
    powerup     : Option<Powerup>,
    shooter     : Option<Shooter>,
    spatial     : Option<Spatial>,
    visual      : Option<Visual>,
}

impl EntityDescriptor {
    fn configure<T>(self: &Self, mut ent: T, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) where T: Builder {
        if let Some(bounding) = &self.bounding {
            let mut bounding_clone = bounding.clone();
            if let Some(faction) = faction {
                bounding_clone.faction = faction;
            }
            ent = ent.with(bounding_clone);
        }
        if let Some(computed) = &self.computed {
            ent = ent.with(computed.clone());
        }
        if let Some(controlled) = &self.controlled {
            ent = ent.with(controlled.clone());
        }
        if let Some(explodes) = &self.explodes {
            ent = ent.with(explodes.clone());
        }
        if let Some(fading) = &self.fading {
            let mut fading_clone = fading.clone();
            fading_clone.start += age;
            fading_clone.end += age;
            ent = ent.with(fading_clone);
        }
        if let Some(hitpoints) = &self.hitpoints {
            ent = ent.with(hitpoints.clone());
        }
        if let Some(inertial) = &self.inertial {
            let mut inertial_clone = inertial.clone();
            if let Some(angle) = angle {
                inertial_clone.v_fraction = Vec2::from(angle);
                inertial_clone.v_current = Vec2::from(angle) * inertial_clone.v_max;
            }
            ent = ent.with(inertial_clone);
        }
        if let Some(lifetime) = &self.lifetime {
            let mut lifetime_clone = lifetime.clone();
            lifetime_clone.0 += age;
            ent = ent.with(lifetime_clone);
        }
        if let Some(powerup) = &self.powerup {
            ent = ent.with(powerup.clone());
        }
        if let Some(shooter) = &self.shooter {
            ent = ent.with(shooter.clone());
        }
        if let Some(spatial) = &self.spatial {
            let mut spatial_clone = spatial.clone();
            if let Some(position) = position {
                spatial_clone.position = position.into();
            }
            if let Some(angle) = angle {
                spatial_clone.angle = angle;
            }
            ent = ent.with(spatial_clone);
        }
        if let Some(visual) = &self.visual {
            ent = ent.with(visual.clone());
        }
        ent.build();
    }
    pub fn spawn_lazy(self: &Self, lazy: &specs::LazyUpdate, entities: &specs::world::EntitiesRes, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        self.configure(LazyBuilder(lazy.create_entity(entities)), age, position, angle, faction);
    }
    pub fn spawn(self: &Self, world: &mut specs::World, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        self.configure(EntityBuilder(world.create_entity()), age, position, angle, faction);
    }
}

trait Builder {
    fn with<T: specs::Component + Send + Sync>(self, c: T) -> Self;
    fn build(self) -> specs::Entity;
}

struct EntityBuilder<'a>(specs::EntityBuilder<'a>);

impl<'a> Builder for EntityBuilder<'a> {
    fn with<T: specs::Component + Send + Sync>(self, c: T) -> Self {
        EntityBuilder(self.0.with(c))
    }
    fn build(self) -> specs::Entity {
        self.0.build()
    }
}

struct LazyBuilder<'a>(specs::world::LazyBuilder<'a>);

impl<'a> Builder for LazyBuilder<'a> {
    fn with<T: specs::Component + Send + Sync>(self, c: T) -> Self {
        LazyBuilder(self.0.with(c))
    }
    fn build(self) -> specs::Entity {
        self.0.build()
    }
}

pub fn sprite_deserialize<'de, D>(deserializer: D) -> Result<Arc<Sprite>, D::Error> where D: Deserializer<'de>, {
    let name = String::deserialize(deserializer)?;
    if let Some(sprite) = unsafe { (*SPRITES).name(&name) } {
        Ok(sprite.clone())
    } else {
        let context = unsafe { &*CONTEXT };
        let sprite = Sprite::from_file(context, &("res/sprite/".to_string() + &name)).unwrap().arc(); // TODO: error handling
        unsafe { (*SPRITES).insert(name.to_string(), sprite.clone()) };
        Ok(sprite)
    }
}

pub fn layer_deserialize<'de, D>(deserializer: D) -> Result<Option<Arc<Layer>>, D::Error> where D: Deserializer<'de>, {
    let name = String::deserialize(deserializer)?;
    if name == "none" {
        Ok(None)
    } else if let Some(layer) = unsafe { (*LAYERS).name(&name) } {
        Ok(Some(layer.clone()))
    } else {
        Err(de::Error::unknown_variant(&name, &[ "<valid layers>" ]))
    }
}

pub fn layer_default() -> Option<Arc<Layer>> {
    None
}

impl<'de> Deserialize<'de> for SpawnerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let name = String::deserialize(deserializer)?;
        if let Some(id) = unsafe { (*SPAWNERS).index_of(&name) } {
            Ok(id)
        } else {
            Err(de::Error::unknown_variant(&name, &[ "<valid loaded spawners>" ]))
        }
    }
}

impl<'de> Deserialize<'de> for FactionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let name = String::deserialize(deserializer)?;
        if let Some(id) = unsafe { (*FACTIONS).iter().position(|x: &String| x == &name) } {
            Ok(FactionId(id))
        } else {
            Err(de::Error::unknown_variant(&name, &[ "<valid loaded factions>" ]))
        }
    }
}