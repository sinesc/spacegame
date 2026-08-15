use crate::prelude::*;
use std::io;
use std::convert::AsRef;
use rodio::Decoder;

pub struct Sound (Arc<Vec<u8>>);

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Sound {
    pub fn load(filename: &str) -> io::Result<Sound> {
        use std::fs::File;
        let mut buf = Vec::new();
        let mut file = File::open(filename)?;
        file.read_to_end(&mut buf)?;
        Ok(Sound(Arc::new(buf)))
    }
    fn cursor(&self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }
    pub fn decoder(&self) -> Decoder<io::Cursor<Sound>> {
        Decoder::try_from(self.cursor()).unwrap()
    }
}
