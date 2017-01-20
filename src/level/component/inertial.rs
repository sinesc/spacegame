use specs;
use radiant_rs::Vec2;

#[derive(Clone, Debug)]
pub struct Inertial {
    /// Maximum velocity
    pub v_max: Vec2,
    /// Fraction of max velocity currently being applied
    pub v_fraction: Vec2,
    /// Computed computed velocity
    pub v_current: Vec2,
    /// Transition speed when trying to move
    pub trans_motion: f32,
    /// Transition speed when not trying to move
    pub trans_rest: f32
}

impl Inertial {
    pub fn new(v_max: Vec2, v_fraction: Vec2, trans_motion: f32, trans_rest: f32) -> Self {
        Inertial {
            v_max       : v_max,
            v_fraction  : v_fraction,
            v_current   : v_max * v_fraction,
            trans_motion: trans_motion,
            trans_rest  : trans_rest,
        }
    }
}

impl specs::Component for Inertial {
    type Storage = specs::VecStorage<Inertial>;
}
