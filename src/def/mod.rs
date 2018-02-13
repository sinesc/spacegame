use prelude::*;
use serde;
use serde_yaml;
use yaml_merge_keys;
use std::iter::FromIterator;

mod layer;
pub use self::layer::*;
mod entity;
pub use self::entity::*;

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
pub fn parse_file<T, F>(filename: &str, mut transform: F) -> Result<T, Error> where T: serde::de::DeserializeOwned, F: FnMut(&mut serde_yaml::Value, Option<&mut serde_yaml::Value>) {

    let mut f = fs::File::open(&filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    parse_str(&contents, &mut transform)
}

/// Parses a directory of yaml files.
pub fn parse_dir<T, F>(source: &str, extensions: &[ &str ], mut transform: F) -> Result<T, Error> where T: serde::de::DeserializeOwned, F: FnMut(&mut serde_yaml::Value, Option<&mut serde_yaml::Value>) {
    
    let files = find(source, extensions)?;
    let mut contents = Vec::new();

    for filename in files {
        let mut f = fs::File::open(&filename)?;
        f.read_to_end(&mut contents)?;
        contents.push('\n' as u8);
    }

    parse_str(&String::from_utf8(contents).unwrap(), &mut transform)
}

fn handle_mapping<F>(mapping: serde_yaml::Mapping, transform: &mut F) -> serde_yaml::Value where F: FnMut(&mut serde_yaml::Value, Option<&mut serde_yaml::Value>) {
    use serde_yaml::Value::*;
    let out_mapping = serde_yaml::Mapping::from_iter(mapping.into_iter().map(|pair| { 
        let (mut key, mut value) = pair;
        match value {            
            Sequence(s) => { value = handle_sequence(s, transform); },
            Mapping(m) => { value = handle_mapping(m, transform); },
            _ => { transform(&mut value, Some(&mut key)); }
        }
        (key, value)
    }));
    serde_yaml::Value::Mapping(out_mapping)
}

fn handle_sequence<F>(sequence: serde_yaml::Sequence, transform: &mut F) -> serde_yaml::Value where F: FnMut(&mut serde_yaml::Value, Option<&mut serde_yaml::Value>) {
    use serde_yaml::Value::*;
    sequence.into_iter().map(|mut item| { 
        match item {            
            Sequence(s) => { item = handle_sequence(s, transform); },
            Mapping(m) => { item = handle_mapping(m, transform); },
            _ => { transform(&mut item, None); }
        }
        item 
    }).collect()
}

fn apply_transform<F>(mut value: serde_yaml::Value, transform: &mut F) -> Result<serde_yaml::Value, Error> where F: FnMut(&mut serde_yaml::Value, Option<&mut serde_yaml::Value>) {
    use serde_yaml::Value::*;
    Ok(match value {
        Sequence(s) => handle_sequence(s, transform),
        Mapping(m) => handle_mapping(m, transform),
        _ => { transform(&mut value, None); value },
    })
}

/// Parses a yaml string.
pub fn parse_str<T, F>(source: &str, transform: &mut F) -> Result<T, Error> where T: serde::de::DeserializeOwned, F: FnMut(&mut serde_yaml::Value, Option<&mut serde_yaml::Value>) {
    match serde_yaml::from_str(source) {
        Ok(value) => {
            match yaml_merge_keys::merge_keys_serde(value) {
                Ok(merged) => { 
                    match apply_transform(merged, transform) {
                        Ok(transformed) => { 
                            Ok(serde_yaml::from_value(transformed).unwrap()) 
                        }
                        Err(error) => Err(Error { description: "Transform failed".to_string(), cause: Some(Box::new(error)) })
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