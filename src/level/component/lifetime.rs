use specs;

#[derive(Clone, Debug)]
pub struct Lifetime(pub f32);

impl specs::Component for Lifetime {
    type Storage = specs::VecStorage<Lifetime>;
}
