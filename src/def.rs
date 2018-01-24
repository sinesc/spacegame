use prelude::*;
use serde;
use serde_json;

#[derive(Deserialize, Debug)]
pub struct Layers {
    pub create: Vec<LayerCreate>,
    pub render: Vec<LayerRender>,
}

#[derive(Deserialize, Debug)]
pub struct LayerCreate {
    pub name: String,
    #[serde(default = "default_scale")]
    pub scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Deserialize, Debug)]
pub struct LayerRender {
    pub name: String,
    pub filter: Option<String>,
}

pub fn parse_layers(filename: &str) -> Result<Layers, Box<Error>> {
    parse(filename)
}

fn parse<T>(filename: &str) -> Result<T, Box<Error>> where T: serde::de::DeserializeOwned {

    let mut f = fs::File::open(&filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    match serde_json::from_str(&contents) {
        Ok(value)  => Ok(value),
        Err(msg)     => Err(Box::<Error + Sync + Send>::from(msg))
    }
}

fn find(source: &str, extensions: &[ &str ]) -> io::Result<Vec<path::PathBuf>> {
    let mut files = Vec::new();
    let entry_set = fs::read_dir(source)?;
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