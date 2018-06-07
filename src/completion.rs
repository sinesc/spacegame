#![allow(dead_code)]

use prelude::*;
use serde::{Deserialize, Deserializer};

/**
 * A data structure that is created in an incomplete state and can later be completed.
 */
pub enum Completion<I, C> {
    Incomplete(Option<Box<I>>),
    Complete(C),
}

impl<I, C> Completion<I, C> {
    pub fn new(value: I) -> Self {
        Completion::Incomplete(Some(Box::new(value)))
    }
    pub fn complete<F>(self: &mut Self, func: F) where F: Fn(I) -> C {
        *self = match self {
            Completion::Incomplete(ref mut opt) => Completion::Complete(func(*opt.take().unwrap())),
            Completion::Complete(_) => panic!("Attempted to complete() already completed Completion"),
        }
    }
    pub fn is_complete(self: &Self) -> bool {
        match self {
            Completion::Incomplete(_) => false,
            Completion::Complete(_) => true
        }
    }
    pub fn is_incomplete(self: &Self) -> bool {
        !self.is_complete()
    }
    pub fn get(self: &Self) -> Option<&C> {
        match self {
            Completion::Incomplete(_) => None,
            Completion::Complete(value) => Some(&value)
        }
    }
}

impl<I, C> Debug for Completion<I, C> where I: Debug, C: Debug {
    fn fmt(self: &Self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Completion::Incomplete(incomplete) => write!(f, "Incomplete({:#?})", incomplete),
            Completion::Complete(complete) => write!(f, "Complete({:#?})", complete)
        }
    }
}

impl<'de, I, C> Deserialize<'de> for Completion<I, C> where I: Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(Completion::new(I::deserialize(deserializer)?))
    }
}