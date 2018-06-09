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
    #[allow(dead_code)]
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


use std::fmt;
use std::marker::PhantomData;
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess};

struct SoundGroupVisitor {
    marker: PhantomData<fn() -> SoundGroup>
}

impl SoundGroupVisitor {
    fn new() -> Self {
        SoundGroupVisitor {
            marker: PhantomData
        }
    }
}

impl<'de> Visitor<'de> for SoundGroupVisitor {

    type Value = SoundGroup;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a soundgroup")
    }

    fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error> where M: SeqAccess<'de> {

        let mut group = SoundGroup { sounds: Vec::new(), rng: ARng::new(0) };

        while let Some(value) = access.next_element()? {
            group.sounds.push(value);
        }

        Ok(group)
    }
}

impl<'de> Deserialize<'de> for SoundGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(SoundGroupVisitor::new())
    }
}

impl<'de> Deserialize<'de> for Sound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let name = String::deserialize(deserializer)?;
        Ok(Sound::load(&("res/sound/".to_string() + &name)).unwrap()) // TODO: error handling
    }
}