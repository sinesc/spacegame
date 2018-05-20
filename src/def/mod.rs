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
pub mod menu;
pub use self::menu::*;

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
                Ok(merged) => {
                    Ok(serde_yaml::from_value(merged).unwrap())
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