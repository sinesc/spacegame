use specs;

/**
 * Exploding component
 * 
 * Entities with this component explode on destruction.
 */
#[derive(Clone, Debug)]
pub struct Exploding {
    pub start_time: f32,     // todo: not yes used. overlay multiple explosions for given duration
    pub duration: f32,
}

impl Exploding {
    pub fn new(start_time: f32, duration: f32) -> Self {
        Exploding {
            start_time: start_time,
            duration: duration,
        }
    }
}

impl specs::Component for Exploding {
    type Storage = specs::VecStorage<Exploding>;
}
