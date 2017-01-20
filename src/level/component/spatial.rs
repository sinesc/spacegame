use specs;
use radiant_rs::Vec2;

#[derive(Clone, Debug)]
pub struct Spatial {
    pub position: Vec2,
    pub angle: f32,
    pub lean: f32,
}

impl Spatial {
    pub fn new(position: Vec2, angle: f32) -> Self {
        Spatial {
            position: position,
            angle   : angle,
            lean    : 0.0,    
        }
    }
}

impl specs::Component for Spatial {
    type Storage = specs::VecStorage<Spatial>;
}
