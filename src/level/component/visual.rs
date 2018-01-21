use specs;
use std::sync::Arc;
use radiant_rs::*;

/**
 * Visual component
 * 
 * Entities with this component are rendered.
 */
#[derive(Clone)]
pub struct Visual {
    pub layer           : Option<Arc<Layer>>,
    pub effect_layer    : Option<Arc<Layer>>,
    pub effect_size     : f32,
    pub sprite          : Arc<Sprite>,
    pub scale           : f32,
    pub color           : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}

impl Visual {
    pub fn new(layer: Option<Arc<Layer>>, effect_layer: Option<Arc<Layer>>, sprite: Arc<Sprite>, color: Color, scale: f32, fps: u32, effect_size: f32) -> Self {
        Visual {
            layer,
            effect_layer,
            effect_size,
            sprite,
            scale,
            color,
            frame_id: 0.0,
            fps,
        }
    }
}

impl specs::Component for Visual {
    type Storage = specs::VecStorage<Visual>;
}
