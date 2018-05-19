use prelude::*;
use ::def::{parse_dir, parse_file, Error};
use level::component::*;
use serde::de::{self, Deserializer, Deserialize};

#[derive(Deserialize, Debug)]
pub struct EntityDef (HashMap<String, EntityItem>);

#[derive(Deserialize, Debug)]
pub struct VisualDescriptor {
    pub layer           : Option<String>,
    pub effect_layer    : Option<String>,
    pub effect_size     : f32,
    pub sprite          : String,
    pub scale           : f32,
    pub color           : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}

#[derive(Deserialize, Debug)]
pub struct EntityItem {
    bounding    : Option<Bounding>,
    exploding   : Option<Exploding>,
    fading      : Option<Fading>,
    hitpoints   : Option<Hitpoints>,
    inertial    : Option<Inertial>,
    lifetime    : Option<Lifetime>,
    shooter     : Option<Shooter>,
    spatial     : Option<Spatial>,
    visual      : Option<VisualDescriptor>,
}

lazy_mut! {
    static mut FACTIONS: Vec<String> = Vec::new();
}

pub fn faction_deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error> where D: Deserializer<'de>, {
    let faction_name = String::deserialize(deserializer)?;
    if let Some(index) = unsafe { FACTIONS.iter().position(|x: &String| x == &faction_name)} {
        Ok(index as u32)
    } else {
        Err(de::Error::unknown_variant(&faction_name, &[ "<valid factions>" ]))
    }
}

pub fn parse_entities() -> Result<EntityDef, Error> {
    unsafe { *FACTIONS = parse_file("res/def/faction.yaml").unwrap(); }
    parse_dir("res/def/entity/", &[ "yaml" ])
}

