use crate::prelude::*;
use super::{parse_file, Error};
use crate::repository::Repository;
use crate::sound::SoundGroup;

pub fn parse_sounds() -> Result<Repository<SoundGroup>, Error> {
    parse_file("res/def/sound.yaml")
}