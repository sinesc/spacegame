use prelude::*;
use specs::DenseVecStorage;

/**
 * Visual component
 *
 * Entities with this component are rendered.
 */
#[derive(Clone, Deserialize, Component)]
pub struct Visual {
    #[serde(deserialize_with = "::def::layer_deserialize")]
    #[serde(default = "::def::layer_default")]
    pub layer           : Option<Arc<Layer>>,
    #[serde(deserialize_with = "::def::layer_deserialize")]
    #[serde(default = "::def::layer_default")]
    pub effect_layer    : Option<Arc<Layer>>,
    pub effect_size     : f32,
    #[serde(deserialize_with = "::def::sprite_deserialize")]
    pub sprite          : Arc<Sprite>,
    pub scale           : f32,
    pub color           : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}

impl Debug for Visual {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Visual")
            .field("layer", &self.layer.is_some())
            .field("effect_layer", &self.effect_layer.is_some())
            .field("effect_size", &self.effect_size)
            .field("sprite", &self.sprite)
            .field("scale", &self.scale)
            .field("color", &self.color)
            .field("frame_id", &self.frame_id)
            .field("fps", &self.fps)
            .finish()
    }
}