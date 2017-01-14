use specs;
use radiant_rs::Vec2;

#[derive(Clone, Debug)]
pub struct Inertial {
    pub v_current: Vec2,
    pub v_max: Vec2,
    pub trans_motion: f32,
    pub trans_rest: f32
}

impl Inertial {
    pub fn new(v_max: Vec2, trans_motion: f32, trans_rest: f32) -> Self {
        Inertial {
            v_current: Vec2(0.0, 0.0),
            v_max: v_max,
            trans_motion: trans_motion,
            trans_rest: trans_rest,
        }
    }
}

impl specs::Component for Inertial {
    type Storage = specs::VecStorage<Inertial>;
}
