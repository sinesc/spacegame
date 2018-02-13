use prelude::*;
use ::def::{parse_dir, parse_file, Error};
use ::level::component::*;

/*
hostile: &hostile
    spatial:
        position: [ 0.0, 0.0 ]
        angle: 0.0
    bounding:
        radius: 1.0
        faction: hostile
    visual:
        layer: base
        effect_layer: effect
        effect_size: 1.0
        sprite: hostile/mine_green_64x64x15.png
        scale: 1.0
        color: [ 1.0, 1.0, 1.0, 1.0 ]
        frame_id: 1
        fps: 30
    hitpoints: 100
*/

#[derive(Deserialize, Debug)]
pub struct EntityDef (HashMap<String, EntityItem>);

#[derive(Deserialize, Debug)]
pub struct EntityItem {
    spatial: Option<Spatial>,
    bounding: Option<Bounding>,
}

#[derive(Deserialize, Debug)]
pub struct LayerCreate {
    pub name: String,
    #[serde(default = "default_scale")]
    pub scale: f32,
    pub blendmode: Option<String>,
}

fn default_scale() -> f32 {
    1.0
}

pub fn parse_entities() -> Result<EntityDef, Error> {
    let factions: Vec<String> = parse_file("res/def/faction.yaml", |_, _, _| {}).unwrap();
    parse_dir("res/def/entity/", &[ "yaml" ], |v, k, p| {
        use serde_yaml::Value::*;
        //println!("{:?}.{:?}: {:?} ", p, k, v);
        
        if p.is_some() && p.unwrap().as_str() == Some(&"bounding") {

            if k.is_some() && k.unwrap().as_str() == Some(&"faction") {
                
                if let Some(index) = factions.iter().position(|&ref x| x == v.as_str().unwrap()) {
                    *v = Number(index.into());
                }
            }

        }
/*
        if let Some(&String(ref p)) = p {
            if p == "bounding" {

                if let Some(&mut String(ref mut k)) = k {
                    if k == "faction" {

                        if let Some(index) = factions.iter().position(|&ref x| x == v.as_str().unwrap()) {
                            *v = Number((index as u32).into());
                        }

                    }
                }

            }
        }
*/        
    })
}

