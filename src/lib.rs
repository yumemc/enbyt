//! A library for Serializing/Deserializing NBT data.
//!
//! NBT Format Reference: <https://minecraft.wiki/w/NBT_format>
pub mod binary;

mod tag;
pub use tag::*;

mod error;
pub use error::*;
