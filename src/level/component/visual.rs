use specs;
use std::sync::Arc;
use radiant_rs::*;

#[derive(Clone)]
pub struct Visual {
    pub layer_id        : Arc<Layer>,
    pub effect_layer_id : Option<Arc<Layer>>,
    pub sprite_id       : Arc<Sprite>,
    pub color           : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}

impl Visual {
    pub fn new(layer_id: Arc<Layer>, effect_layer_id: Option<Arc<Layer>>, sprite_id: Arc<Sprite>, color: Color, fps: u32) -> Self {
        Visual {
            layer_id    : layer_id,
            effect_layer_id    : effect_layer_id,
            sprite_id   : sprite_id,
            color       : color,
            frame_id    : 0.0,
            fps         : fps,
        }
    }
}

impl specs::Component for Visual {
    type Storage = specs::VecStorage<Visual>;
}
