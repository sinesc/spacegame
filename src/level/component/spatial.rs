use specs;
use radiant_rs::*;

#[derive(Clone, Debug)]
pub struct Spatial {
    pub position: Point2,
    pub angle: Angle,
    pub lean: f32,
    pub angle_locked: bool,
}

impl Spatial {
    pub fn new(position: Vec2, angle: Angle, angle_locked: bool) -> Self {
        Spatial {
            position    : position,
            angle       : angle,
            lean        : 0.0,
            angle_locked: angle_locked,
        }
    }
}

impl specs::Component for Spatial {
    type Storage = specs::VecStorage<Spatial>;
}
