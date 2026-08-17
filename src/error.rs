use std::io;

use thiserror::Error;

use crate::Tag;

#[derive(Error, Debug)]
pub enum NBTError {
    #[error("invalid tag name {0:?}")]
    InvalidTagName(Option<String>),

    #[error("item does not match expected payload type: {0:?}")]
    UnexpectedType(u8, Tag),

    #[error("not all entries have the same key as their value's name")]
    InconsistentCompound,

    #[error("not all list items have specified type")]
    InconsistentList,

    #[error("invalid payload type")]
    InvalidPayloadType,

    #[error("io error: {0:?}")]
    IO(#[from] io::Error),
}
