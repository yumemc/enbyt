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

impl PartialEq for NBTError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NBTError::InvalidTagName(a), NBTError::InvalidTagName(b)) => a == b,
            (NBTError::UnexpectedType(a_type, a_tag), NBTError::UnexpectedType(b_type, b_tag)) => {
                a_type == b_type && a_tag == b_tag
            }
            (NBTError::InconsistentCompound, NBTError::InconsistentCompound) => true,
            (NBTError::InconsistentList, NBTError::InconsistentList) => true,
            (NBTError::InvalidPayloadType, NBTError::InvalidPayloadType) => true,

            // not sure if it's correct but probably best we can do
            (NBTError::IO(a), NBTError::IO(b)) => a.kind() == b.kind(),

            _ => false,
        }
    }
}
