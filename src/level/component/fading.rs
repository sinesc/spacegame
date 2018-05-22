use specs::DenseVecStorage;

/**
 * Fading component
 *
 * Entities with this component fade after a certain amount of time.
 */
#[derive(Clone, Debug, Deserialize, Default, Component)]
pub struct Fading {
    //pub value: f32,
    pub start: f32,
    pub end: f32,
}