use prelude::*;
use super::{parse_file, Error};
use std::ops::Deref;

pub fn parse_menu() -> Result<MenuDef, Error> {
    parse_file("res/def/menu.yaml")
}

#[derive(Deserialize, Debug)]
pub struct MenuDef (HashMap<String, MenuGroup>);

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct MenuGroup {
    pub items   : Vec<MenuItem>,
    pub left    : f32,
    pub top     : f32,
    pub stride_x: f32,
    pub stride_y: f32,
}

impl Default for MenuGroup {
    fn default() -> MenuGroup {
        MenuGroup {
            items   : Vec::new(),
            left    : 0.0,
            top     : 0.0,
            stride_x: 0.0,
            stride_y: 0.1,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct MenuItem {
    pub label   : String,
    pub action  : String,
    pub stride_x: Option<f32>,
    pub stride_y: Option<f32>,

}

impl Deref for MenuDef {
    type Target = HashMap<String, MenuGroup>;

    fn deref(&self) -> &HashMap<String, MenuGroup> {
        &self.0
    }
}