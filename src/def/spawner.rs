use prelude::*;
use ::def::{parse_dir, Error};
use repository::Repository;

pub fn parse_spawners() -> Result<Repository<SpawnerDescriptor, SpawnerId>, Error> {
    parse_dir("res/def/spawner/", &[ "yaml" ])
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
    pub entity: Option<String>,
    pub sound: Option<String>,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct SpawnerDescriptor {
    pub dispatch: SpawnerDispatch,
    pub entities: Vec<SpawnerParameters>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SpawnerId(pub usize);

impl From<SpawnerId> for usize {
    fn from(input: SpawnerId) -> usize {
        input.0
    }
}

impl From<usize> for SpawnerId {
    fn from(input: usize) -> SpawnerId {
        SpawnerId(input)
    }
}