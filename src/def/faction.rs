use prelude::*;
use super::{parse_file, Error};

pub fn parse_factions() -> Result<Vec<String>, Error> {
    parse_file("res/def/faction.yaml")
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct FactionId(pub usize);

impl From<FactionId> for usize {
    fn from(input: FactionId) -> usize {
        input.0
    }
}

impl From<usize> for FactionId {
    fn from(input: usize) -> FactionId {
        FactionId(input)
    }
}