use crate::prelude::*;

/**
 * Visual component
 *
 * Entities with this component are rendered.
 */
#[derive(Clone, Debug)]
pub struct Visual {
    pub layer           : Option<Arc<Layer>>,
    pub effect_layer    : Option<Arc<Layer>>,
    pub sprite          : Arc<Sprite>,
    pub scale           : f32,
    pub effect_scale    : f32,
    pub color           : Color,
    pub effect_color    : Color,
    pub frame_id        : f32,
    pub fps             : u32,
}
