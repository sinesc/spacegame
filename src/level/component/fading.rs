use specs;

/**
 * Fading component
 * 
 * Entities with this component fade after a certain amount of time.
 */
#[derive(Clone, Debug)]
pub struct Fading {
    //pub value: f32,
    pub start: f32,
    pub end: f32,
}

impl Fading {
    pub fn new(start: f32, end: f32) -> Self {
        Fading {
            start   : start,
            end     : end,
        }
    }
}

impl specs::Component for Fading {
    type Storage = specs::VecStorage<Fading>;
}
