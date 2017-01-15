use specs;
use radiant_rs::scene::*;

#[derive(Clone)]
pub struct Visual {
    pub layer_id: LayerId,
    pub sprite_id: SpriteId,
    pub frame_id: u32,
}

impl Visual {
    pub fn new(layer_id: LayerId, sprite_id: SpriteId) -> Self {
        Visual { layer_id: layer_id, sprite_id: sprite_id, frame_id: 0 }
    }
}

impl specs::Component for Visual {
    type Storage = specs::VecStorage<Visual>;
}
