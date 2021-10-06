#[cfg(feature = "build")]
pub mod builder;

pub mod reader;
pub mod scores;
pub mod search;
pub mod structs;
pub mod writer;

pub const VERSION_STRING: &str = "stork-3";
