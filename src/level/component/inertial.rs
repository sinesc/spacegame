#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use specs;
use radiant_rs::Vec2;

#[derive(Clone, Debug)]
pub struct Inertial {
    pub velocity: Vec2<f32>,
}

impl Inertial {
    pub fn new() -> Self {
        Inertial { velocity: Vec2::<f32>(0.0, 0.0) }
    }
}

impl specs::Component for Inertial {
    type Storage = specs::VecStorage<Inertial>;
}
