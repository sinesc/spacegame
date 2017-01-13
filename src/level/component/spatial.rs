#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use specs;
use radiant_rs::Vec2;

#[derive(Clone, Debug)]
pub struct Spatial {
    pub pos: Vec2<f32>,
    pub dir: f32,
}

impl Spatial {
    pub fn new(pos: Vec2<f32>, dir: f32) -> Self {
        Spatial { pos: pos, dir: dir }
    }
}

impl specs::Component for Spatial {
    type Storage = specs::VecStorage<Spatial>;
}
