use crate::prelude::*;
use crate::repository::Repository;

// Ugly unsafe global: gives the scripting spawn code access to the layer
// repository (the radiant Context can't carry arbitrary state).
// Pointed at Infrastructure.layer by Level::new.
pub static mut LAYERS: *const Repository<Arc<Layer>> = 0 as _;
