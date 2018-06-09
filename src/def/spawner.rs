use prelude::*;
use ::def::{parse_dir, yaml_merge_maps, Error, EntityDescriptor};
use completion::Completion;
use repository::Repository;
use serde_yaml;

pub fn parse_spawners() -> Result<Repository<SpawnerDescriptor, SpawnerId>, Error> {
    parse_dir("res/def/spawner/", &[ "yaml" ])
}

/// Completes custom entity definitions on spawners (merges the base entity). This is called by parse_entities().
pub fn complete_spawners(spawners: &mut Repository<SpawnerDescriptor, SpawnerId>, entities: &HashMap<String, serde_yaml::Value>) {
    for spawner in spawners.values_mut() {
        for spawn in &mut spawner.entities {
            let base_entity = entities.get(&spawn.base).expect(&format!("Spawner-entity {} is not defined.", &spawn.base));
            if let Some(ref mut extension) = &mut spawn.extend {
                extension.complete(|mut incomplete| {
                    // merge base entity yaml map into spawner entity, then deserialize the result
                    yaml_merge_maps(&mut incomplete, &base_entity);
                    serde_yaml::from_value(incomplete).unwrap()
                });
            } // TODO: use else here once rust if else scoping bug is fixed
            if spawn.extend.is_none() {
                let entity = serde_yaml::from_value(base_entity.clone()).unwrap();
                spawn.extend = Some(Completion::completed(entity));
            }
        }
    }
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

#[derive(Default, Debug, Deserialize)]
pub struct SpawnerParameters {
    pub position: Vec2,
    pub angle: Angle,
    pub base: String,
    pub extend: Option<Completion<serde_yaml::Value, EntityDescriptor>>,
    pub sound: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
pub struct SpawnerDescriptor {
    pub dispatch: SpawnerDispatch,
    pub entities: Vec<SpawnerParameters>, // TODO: rename. maybe items or spawns?
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