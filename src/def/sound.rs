use prelude::*;
use super::{parse_file, Error};
use repository::Repository;
use sound::SoundGroup;

pub fn parse_sounds() -> Result<Repository<SoundGroup>, Error> {
    parse_file("res/def/sound.yaml")
}