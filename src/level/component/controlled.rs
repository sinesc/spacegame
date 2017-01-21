use specs;

#[derive(Clone, Debug)]
pub struct Controlled {
    pub input_id: u32,
    pub av_current: f32,
    pub av_trans: f32,
    pub av_max: f32,
}

impl Controlled {
    pub fn new(input_id: u32) -> Self {
        Controlled {
            input_id: input_id,
            av_current: 0.0,
            av_trans: 8.0,
            av_max: 5.0,
        }
    }
}

impl specs::Component for Controlled {
    type Storage = specs::HashMapStorage<Controlled>;
}
