use serde::{Deserialize, Deserializer};
pub use radiant_utils::util::Periodic;
pub use radiant_utils::maths::{Angle, Vec2};
pub use radiant::Color;
use ::def::{parse_file, Error};

pub fn parse_factions() -> Result<Vec<String>, Error> {
    parse_file("res/def/faction.yaml")
}

pub fn periodic_deserialize<'de, D>(deserializer: D) -> Result<Periodic, D::Error> where D: Deserializer<'de>, {
    Ok(Periodic::new(0.0, f32::deserialize(deserializer)?))
}

pub fn periodic_default() -> Periodic {
    Periodic::new(0.0, 1.0)
}
