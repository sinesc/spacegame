use serde::de::{self, Deserializer, Deserialize};
use specs;
use prelude::*;
use def::{parse_dir, Error};
use level::component::*;
use repository::Repository;
use def::spawner::*;

static mut FACTIONS: *const Vec<String> = 0 as _;
static mut SPRITES: *const Repository<String, Arc<Sprite>> = 0 as _;
static mut LAYERS: *const Repository<String, Arc<Layer>> = 0 as _;
static mut SPAWNERS: *const Repository<String, SpawnerDescriptor> = 0 as _;

pub fn parse_entities(factions: &Vec<String>, spawners: &Repository<String, SpawnerDescriptor>, sprites: &Repository<String, Arc<Sprite>>, layers: &Repository<String, Arc<Layer>>) -> Result<Repository<String, EntityDescriptor>, Error> {
    unsafe {
        // set up some ugly unsafe global state to work around missing DeserializeSeed in Serde-Yaml
        FACTIONS = factions as *const Vec<String>;
        SPRITES = sprites as *const Repository<String, Arc<Sprite>>;
        LAYERS = layers as *const Repository<String, Arc<Layer>>;
        SPAWNERS = spawners as *const Repository<String, SpawnerDescriptor>;
    }
    parse_dir("res/def/entity/", &[ "yaml" ])
}

#[derive(Deserialize, Debug)]
pub struct EntityDescriptor {
    bounding    : Option<Bounding>,
    computed    : Option<Computed>,
    controlled  : Option<Controlled>,
    exploding   : Option<Exploding>,
    fading      : Option<Fading>,
    hitpoints   : Option<Hitpoints>,
    inertial    : Option<Inertial>,
    lifetime    : Option<Lifetime>,
    shooter     : Option<Shooter>,
    spatial     : Option<Spatial>,
    visual      : Option<Visual>,
}

pub trait Builder {
    fn with<T: specs::Component + Send + Sync>(self, c: T) -> Self;
    fn build(self) -> specs::Entity;
}

pub struct EntityBuilder<'a>(specs::EntityBuilder<'a>);

impl<'a> Builder for EntityBuilder<'a> {
    fn with<T: specs::Component + Send + Sync>(self, c: T) -> Self {
        EntityBuilder(self.0.with(c))
    }
    fn build(self) -> specs::Entity {
        self.0.build()
    }
}

pub struct LazyBuilder<'a>(specs::world::LazyBuilder<'a>);

impl<'a> Builder for LazyBuilder<'a> {
    fn with<T: specs::Component + Send + Sync>(self, c: T) -> Self {
        LazyBuilder(self.0.with(c))
    }
    fn build(self) -> specs::Entity {
        self.0.build()
    }
}

impl EntityDescriptor {
    fn configure<T>(self: &Self, mut ent: T, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<u32>) where T: Builder {
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
        if let Some(exploding) = &self.exploding {
            ent = ent.with(exploding.clone());
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
                inertial_clone.v_fraction = angle.to_vec2();
                inertial_clone.v_current = angle.to_vec2() * inertial_clone.v_max;
            }
            ent = ent.with(inertial_clone);
        }
        if let Some(lifetime) = &self.lifetime {
            let mut lifetime_clone = lifetime.clone();
            lifetime_clone.0 += age;
            ent = ent.with(lifetime_clone);
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
    pub fn spawn_lazy(self: &Self, lazy: &specs::LazyUpdate, entities: &specs::world::EntitiesRes, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<u32>) {
        self.configure(LazyBuilder(lazy.create_entity(entities)), age, position, angle, faction);
    }
    pub fn spawn(self: &Self, world: &mut specs::World, age: f32, position: Option<Vec2>, angle: Option<Angle>, faction: Option<u32>) {
        self.configure(EntityBuilder(world.create_entity()), age, position, angle, faction);
    }
}

pub fn faction_deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error> where D: Deserializer<'de>, {
    let name = String::deserialize(deserializer)?;
    if let Some(index) = unsafe { (*FACTIONS).iter().position(|x: &String| x == &name) } {
        Ok(index as u32)
    } else {
        Err(de::Error::unknown_variant(&name, &[ "<valid factions>" ]))
    }
}

pub fn sprite_deserialize<'de, D>(deserializer: D) -> Result<Arc<Sprite>, D::Error> where D: Deserializer<'de>, {
    let name = String::deserialize(deserializer)?;
    if let Some(sprite) = unsafe { (*SPRITES).name(&name) } {
        Ok(sprite.clone())
    } else {
        Err(de::Error::unknown_variant(&name, &[ "<valid loaded sprites>" ]))
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

pub fn spawner_deserialize<'de, D>(deserializer: D) -> Result<usize, D::Error> where D: Deserializer<'de>, {
    let name = String::deserialize(deserializer)?;
    if let Some(spawner_id) = unsafe { (*SPAWNERS).index_of(&name) } {
        Ok(spawner_id)
    } else {
        Err(de::Error::unknown_variant(&name, &[ "<valid loaded spawners>" ]))
    }
}

pub fn layer_default() -> Option<Arc<Layer>> {
    None
}
