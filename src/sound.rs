use prelude::*;
use std::io;
use std::convert::AsRef;
use rodio;
use rodio::{Decoder, Source, Sample};
use rodio::source::SamplesConverter;
use radiant_utils::util::ARng;

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
    fn cursor(self: &Self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }
    fn decoder(self: &Self) -> Decoder<io::Cursor<Sound>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }
    pub fn samples<T>(self: &Self) -> SamplesConverter<Decoder<io::Cursor<Sound>>, T> where T: Sample {
        self.decoder().convert_samples()
    }
}

pub struct SoundGroup {
    sounds: Vec<Sound>,
    rng: ARng,
}

impl SoundGroup {
    pub fn load(filenames: &[&str]) -> io::Result<SoundGroup> {
        let sounds: io::Result<Vec<_>> = filenames.iter().map(|filename| { Sound::load(filename) }).collect();
        Ok(SoundGroup {
            sounds: sounds?,
            rng: ARng::new(0),
        })
    }
    fn decoder(self: &Self) -> Decoder<io::Cursor<Sound>> {
        self.rng.chose(&self.sounds).decoder()
    }
    pub fn samples<T>(self: &Self) -> SamplesConverter<Decoder<io::Cursor<Sound>>, T> where T: Sample {
        self.decoder().convert_samples()
    }
}
