use serde::de::{self, Deserializer, Deserialize};
use hecs;
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
static mut CONTEXT: *const Context = 0 as _;

pub fn parse_entities(context: &Context, sprites: &mut Repository<Arc<Sprite>>, factions: &Vec<String>, spawners: &mut Repository<SpawnerDescriptor, SpawnerId>, layers: &Repository<Arc<Layer>>) -> Result<Repository<EntityDescriptor>, Error> {
    unsafe {
        FACTIONS = factions as *const Vec<String>;
        SPRITES = sprites as *mut Repository<Arc<Sprite>>;
        LAYERS = layers as *const Repository<Arc<Layer>>;
        SPAWNERS = spawners as *const Repository<SpawnerDescriptor, SpawnerId>;
        CONTEXT = context as *const Context;
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
    fn configure(&self, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) -> hecs::EntityBuilder {
        let mut b = hecs::EntityBuilder::new();
        if let Some(bounding) = &self.bounding {
            let mut c = bounding.clone();
            if let Some(faction) = faction { c.faction = faction; }
            b.add(c);
        }
        if let Some(c) = &self.computed { b.add(c.clone()); }
        if let Some(c) = &self.controlled { b.add(c.clone()); }
        if let Some(c) = &self.explodes { b.add(c.clone()); }
        if let Some(fading) = &self.fading {
            let mut c = fading.clone();
            c.start += age;
            c.end += age;
            b.add(c);
        }
        if let Some(c) = &self.hitpoints { b.add(c.clone()); }
        if let Some(inertial) = &self.inertial {
            let mut c = inertial.clone();
            if let Some(angle) = angle {
                c.v_fraction = Vec2::from(angle);
                c.v_current = Vec2::from(angle) * c.v_max;
            }
            b.add(c);
        }
        if let Some(lifetime) = &self.lifetime {
            let mut c = lifetime.clone();
            c.0 += age;
            b.add(c);
        }
        if let Some(c) = &self.powerup { b.add(c.clone()); }
        if let Some(c) = &self.shooter { b.add(c.clone()); }
        if let Some(spatial) = &self.spatial {
            let mut c = spatial.clone();
            if let Some(position) = position { c.position = position.into(); }
            if let Some(angle) = angle { c.angle = angle; }
            b.add(c);
        }
        if let Some(c) = &self.visual { b.add(c.clone()); }
        b
    }

    pub fn spawn_lazy(&self, cmd: &mut hecs::CommandBuffer, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        let mut b = self.configure(age, position, angle, faction);
        cmd.spawn(b.build());
    }

    pub fn spawn(&self, world: &mut hecs::World, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<FactionId>) {
        let mut b = self.configure(age, position, angle, faction);
        world.spawn(b.build());
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
