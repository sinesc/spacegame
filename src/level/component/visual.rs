use prelude::*;
use specs::DenseVecStorage;

/**
 * Visual component
 *
 * Entities with this component are rendered.
 */
#[derive(Clone, Debug, Deserialize, Component)]
pub struct Visual {
    #[serde(deserialize_with = "::def::layer_deserialize")]
    #[serde(default = "::def::layer_default")]
    pub layer           : Option<Arc<Layer>>,
    #[serde(deserialize_with = "::def::layer_deserialize")]
    #[serde(default = "::def::layer_default")]
    pub effect_layer    : Option<Arc<Layer>>,
    #[serde(deserialize_with = "::def::sprite_deserialize")]
    pub sprite          : Arc<Sprite>,
    pub scale           : f32,
    pub effect_scale    : f32,
    pub color           : Color,
    pub effect_color    : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}
