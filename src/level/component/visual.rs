use specs;
use radiant_rs::*;
use radiant_rs::scene::*;

#[derive(Clone)]
pub struct Visual {
    pub layer_id    : LayerId,
    pub sprite_id   : SpriteId,
    pub color       : Color,
    pub frame_id    : f32,
    pub fps         : u32,
}

impl Visual {
    pub fn new(layer_id: LayerId, sprite_id: SpriteId, color: Color, fps: u32) -> Self {
        Visual {
            layer_id    : layer_id,
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
