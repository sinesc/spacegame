use specs;

#[derive(Clone, Debug)]
pub struct Explosion {
    pub start_time: f32,     // !todo not yes used. overlay multiple explosions for given duration
    pub duration: f32,
}

impl Explosion {
    pub fn new(start_time: f32, duration: f32) -> Self {
        Explosion {
            start_time: start_time,
            duration: duration,
        }
    }
}

impl specs::Component for Explosion {
    type Storage = specs::VecStorage<Explosion>;
}
