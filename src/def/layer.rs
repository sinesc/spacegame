use prelude::*;
use super::{parse_file, Error};

pub fn parse_layers() -> Result<LayerDef, Error> {
    parse_file("res/def/layer.yaml", |ref v, k| {})
}

#[derive(Deserialize, Debug)]
pub struct LayerDef {
    pub create: Vec<LayerCreate>,
    pub render: Vec<LayerRender>,
}

#[derive(Deserialize, Debug)]
pub struct LayerCreate {
    pub name: String,
    #[serde(default = "default_scale")]
    pub scale: f32,
    pub blendmode: Option<String>,
}

fn default_scale() -> f32 { 1.0 }

#[derive(Deserialize, Debug)]
pub struct LayerRender {
    pub name: String,
    pub filter: Option<String>,
    #[serde(default = "default_component")]
    pub component: u32,
}

fn default_component() -> u32 { 1 }