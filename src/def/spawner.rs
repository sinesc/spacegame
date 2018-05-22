use prelude::*;
use ::def::{parse_dir, Error};
use repository::Repository;

pub fn parse_spawners() -> Result<Repository<String, SpawnerDescriptor>, Error> {
    parse_dir("res/def/spawn/", &[ "yaml" ])
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum SpawnerDispatch {
    All,
    Index,
    BitMask
}

impl Default for SpawnerDispatch {
    fn default() -> SpawnerDispatch {
        SpawnerDispatch::All
    }
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct SpawnerParameters {
    pub position: Vec2,
    pub angle: Angle,
    pub entity: String,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct SpawnerDescriptor {
    pub dispatch: SpawnerDispatch,
    pub entities: Vec<SpawnerParameters>,
}