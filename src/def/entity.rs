use prelude::*;
use ::def::{parse_dir, Error};
use level::component::*;
use serde::de::{self, Deserializer, Deserialize};
use specs;

static mut FACTIONS: *const Vec<String> = 0 as _;
static mut SPRITES: *const HashMap<String, Arc<Sprite>> = 0 as _;
static mut LAYERS: *const HashMap<String, Arc<Layer>> = 0 as _;

pub fn parse_entities(factions: &Vec<String>, sprites: &HashMap<String, Arc<Sprite>>, layers: &HashMap<String, Arc<Layer>>) -> Result<HashMap<String, EntityDescriptor>, Error> {
    unsafe {
        // set up some ugly unsafe global state to work around missing DeserializeSeed in Serde-Yaml
        FACTIONS = factions as *const Vec<String>;
        SPRITES = sprites as *const HashMap<String, Arc<Sprite>>;
        LAYERS = layers as *const HashMap<String, Arc<Layer>>;
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

impl EntityDescriptor {
    pub fn spawn<T>(self: &Self, world: &mut specs::World, position: T) where T: Into<Vec2> {

        let mut ent = world.create_entity();

        if let Some(bounding) = &self.bounding { // todo: use rtti here
            ent = ent.with(bounding.clone());
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
            ent = ent.with(fading.clone());
        }
        if let Some(hitpoints) = &self.hitpoints {
            ent = ent.with(hitpoints.clone());
        }
        if let Some(inertial) = &self.inertial {
            ent = ent.with(inertial.clone());
        }
        if let Some(lifetime) = &self.lifetime {
            ent = ent.with(lifetime.clone());
        }
        if let Some(shooter) = &self.shooter {
            ent = ent.with(shooter.clone());
        }
        if let Some(spatial) = &self.spatial {
            let mut spatial_clone = spatial.clone();
            spatial_clone.position = position.into();
            ent = ent.with(spatial_clone);
        }
        if let Some(visual) = &self.visual {
            ent = ent.with(visual.clone());
        }

        ent.build();
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
    if let Some(sprite) = unsafe { (*SPRITES).get(&name) } {
        Ok(sprite.clone())
    } else {
        Err(de::Error::unknown_variant(&name, &[ "<valid loaded sprites>" ]))
    }
}

pub fn layer_deserialize<'de, D>(deserializer: D) -> Result<Option<Arc<Layer>>, D::Error> where D: Deserializer<'de>, {
    let name = String::deserialize(deserializer)?;
    if let Some(layer) = unsafe { (*LAYERS).get(&name) } {
        Ok(Some(layer.clone()))
    } else {
        Err(de::Error::unknown_variant(&name, &[ "<valid layers>" ]))
    }
}

pub fn layer_default() -> Option<Arc<Layer>> {
    None
}
