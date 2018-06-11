use prelude::*;
use serde;
use serde_yaml;
use yaml_merge_keys;

pub mod misc;
pub use self::misc::*;
pub mod layer;
pub use self::layer::*;
pub mod entity;
pub use self::entity::*;
pub mod spawner;
pub use self::spawner::*;
pub mod menu;
pub use self::menu::*;
pub mod faction;
pub use self::faction::*;
pub mod sound;
pub use self::sound::*;

lazy_static! {
    static ref MERGE_KEY: serde_yaml::Value = serde_yaml::Value::String("<<<".to_string());
}

#[derive(Debug)]
pub struct Error {
    description: String,
    cause: Option<Box<error::Error>>,
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Error { description: "IO Error".to_string(), cause: Some(Box::new(source)) }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for Error {
    fn description(self: &Self) -> &str {
        &self.description
    }
    fn cause(self: &Self) -> Option<&error::Error> {
        self.cause.as_ref().map(|cause| &**cause)
    }
}

/// Parses a single yaml file.
fn parse_file<T>(filename: &str) -> Result<T, Error> where T: serde::de::DeserializeOwned {

    let mut f = fs::File::open(&filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    parse_str(&contents)
}

/// Parses a directory of yaml files.
fn parse_dir<T>(source: &str, extensions: &[ &str ]) -> Result<T, Error> where T: serde::de::DeserializeOwned {

    let files = find(source, extensions)?;
    let mut contents = Vec::new();

    for filename in files {
        let mut f = fs::File::open(&filename)?;
        f.read_to_end(&mut contents)?;
        contents.push('\n' as u8);
    }

    parse_str(&String::from_utf8(contents).unwrap())
}

/// Parses a yaml string.
fn parse_str<T>(source: &str) -> Result<T, Error> where T: serde::de::DeserializeOwned {
    match serde_yaml::from_str(source) {
        Ok(value) => {
            match yaml_merge_keys::merge_keys_serde(value) {
                Ok(mut merged) => {
                    yaml_merge_keys_recursively(&mut merged);
                    match serde_yaml::from_value(merged) {
                        Ok(result) => Ok(result),
                        Err(error) => Err(Error { description: "Deserialize failed". to_string(), cause: Some(Box::new(error)) })
                    }
                }
                Err(error) => Err(Error { description: "Merge failed".to_string(), cause: Some(Box::new(error)) })
            }
        }
        Err(error) => Err(Error { description: "Parsing failed".to_string(), cause: Some(Box::new(error)) })
    }
}

/// Returns a list of files with the given extension in the given path.
fn find(path: &str, extensions: &[ &str ]) -> io::Result<Vec<path::PathBuf>> {
    let mut files = Vec::new();
    let entry_set = fs::read_dir(path)?;
    let mut entries = entry_set.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|a, b| a.path().cmp(&b.path()));
    for entry in entries {
        let extension = entry.path().extension().map_or("", |p| p.to_str().unwrap()).to_string(); // !todo better solution or intended user experience ?
        if extensions.iter().find(|ext| **ext == extension).is_some() && entry.path().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

/// Merges values of "<<<" keys into the current map.
pub fn yaml_merge_keys_recursively(value: &mut serde_yaml::Value) {
    use serde_yaml::Value;

    // check for merge key, if present, merge and remove it

    let mut merge_data = None;

    if let Value::Mapping(mapping) = value {
        if let Some(mut ref_value) = mapping.remove(&MERGE_KEY) {
            // ensure refs in the merge source are resolved
            yaml_merge_keys_recursively(&mut ref_value);
            // TODO: ugly method to get around active borrow on value
            merge_data = Some(ref_value);
        }
    }

    if let Some(merge_data) = merge_data {
        yaml_merge_maps(value, &merge_data);
    }

    // recurse

    if let Value::Mapping(mapping) = value {
        for (_, v) in mapping {
            yaml_merge_keys_recursively(v);
        }
    } else if let Value::Sequence(sequence) = value {
        for v in sequence {
            yaml_merge_keys_recursively(v);
        }
    }
}

/// Merges keys from source into destination, if they don't already exist. Handles maps recursively.
pub fn yaml_merge_maps(destination: &mut serde_yaml::Value, source: &serde_yaml::Value) {
    use serde_yaml::Value;

    if let Value::Mapping(source_map) = source {
        if let Value::Mapping(destination_map) = destination {
            for (k, v) in source_map.iter() {
                if !destination_map.contains_key(k) {
                    destination_map.insert(k.clone(), v.clone());
                } else if destination_map[k].is_mapping() {
                    yaml_merge_maps(&mut destination_map[k], &source_map[k]);
                }
            }
        } else {
            panic!("destination is not a map"); // TODO: Result?
        }
    }
}