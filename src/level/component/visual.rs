use specs;
use std::sync::Arc;
use radiant_rs::*;

#[derive(Clone)]
pub struct Visual {
    pub layer           : Arc<Layer>,
    pub effect_layer    : Option<Arc<Layer>>,
    pub sprite          : Arc<Sprite>,
    pub color           : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}

impl Visual {
    pub fn new(layer: Arc<Layer>, effect_layer: Option<Arc<Layer>>, sprite: Arc<Sprite>, color: Color, fps: u32) -> Self {
        Visual {
            layer           : layer,
            effect_layer    : effect_layer,
            sprite          : sprite,
            color           : color,
            frame_id        : 0.0,
            fps             : fps,
        }
    }
}

impl specs::Component for Visual {
    type Storage = specs::VecStorage<Visual>;
}
