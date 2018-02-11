pub use std::{fs, cmp, path, collections, io, process, fmt, error};
pub use std::io::prelude::*;
pub use std::result::Result;
pub use std::collections::HashMap;
pub use std::sync::Arc;
pub use std::time::Instant;
pub use std::f32::consts::PI;
pub use radiant_utils::maths as rm;
pub use radiant_utils::util as ru;
pub use radiant_utils::maths::{Angle, Vec2, approach, min, max};
pub use radiant_utils::util::{Periodic, Rng};
pub use radiant_utils::loops::renderloop;
pub use radiant::{Layer, Sprite, Color, Input, InputId, Font, Renderer, Texture, RenderContext, Display, blendmodes, postprocessors};
pub use super::def;

// These are only used to circumvent the crazy orphan rules.
#[derive(Deserialize)]
#[serde(remote = "Angle")]
pub struct AngleOrphan<T = f32>(pub T);

#[derive(Deserialize)]
#[serde(remote = "Vec2")]
pub struct Vec2Orphan<T = f32>(pub T, pub T);