use specs;
use radiant_rs::*;

#[derive(Clone, Debug)]
pub struct Spatial {
    pub position: Vec2,
    pub angle: Angle,
    pub lean: f32,
}

impl Spatial {
    pub fn new(position: Vec2, angle: Angle) -> Self {
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
