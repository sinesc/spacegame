use specs;
use radiant_rs::*;

#[derive(Clone, Debug)]
pub struct Explosion {
    pub star_time: f32,     // !todo not yes used. overlay multiple explosions for given duration
    pub duration: f32,
}

impl Explosion {
    pub fn new(star_time: f32, duration: f32) -> Self {
        Explosion {
            star_time: star_time,
            duration: duration,
        }
    }
}

impl specs::Component for Explosion {
    type Storage = specs::VecStorage<Explosion>;
}
