use prelude::*;
use std::io;
use std::convert::AsRef;
use rodio;

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
    pub fn cursor(self: &Self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }
    pub fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<Sound>> {
        rodio::Decoder::new(self.cursor()).unwrap()
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
    pub fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<Sound>> {
        rodio::Decoder::new(self.rng.chose(&self.sounds).cursor()).unwrap()
    }
}


//TMP

use std::sync::atomic::{AtomicUsize, Ordering};

/// A very simple, seedable atomic random number generator based on sin().
pub struct ARng (AtomicUsize);

impl ARng {

    /// Creates a new instance with given seed.
    pub fn new(seed: usize) -> ARng {
        ARng(AtomicUsize::new(seed))
    }

    /// Returns a random number between 0.0 and non-inclusive 1.0
    pub fn get(self: &Self) -> f64 {
        let pos = self.0.fetch_add(1, Ordering::SeqCst);
        let large = (pos as f64).sin() * 100000000.0;
        large - large.floor()
    }

    /// Returns a random number between min and non-inclusive max.
    pub fn range(self: &Self, min: f64, max: f64) -> f64 {
        let pos = self.0.fetch_add(1, Ordering::SeqCst);
        let large = (pos as f64).sin() * 100000000.0;
        let base = (large - large.floor()) as f64;
        min + base * (max - min)
    }

    /// Returns a random item from given slice.
    pub fn chose<'a, T>(self: &Self, source: &'a [ T ]) -> &'a T {
        let index = self.range(0 as f64, source.len() as f64) as usize;
        &source[index]
    }
}