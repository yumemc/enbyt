use std::io;

use thiserror::Error;
use winnow::error::{ContextError, ErrMode};

use crate::{Tag, TagPayloadType};

#[derive(Error, Debug)]
pub enum NBTError {
    #[error("invalid tag name {0:?}")]
    InvalidTagName(Option<String>),

    #[error("item does not match expected payload type: {0:?}")]
    UnexpectedType(u8, Tag),

    #[error("not all list items have specified type")]
    InconsistentList,

    #[error("payload type {0:?} cannot be used for list elements")]
    InvalidListElementType(TagPayloadType),

    #[error("invalid payload type")]
    InvalidPayloadType,

    #[error("io error: {0:?}")]
    IO(#[from] io::Error),

    #[error("parsing error: {0:?}")]
    ParsingError(ErrMode<ContextError>),
}

impl PartialEq for NBTError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NBTError::InvalidTagName(a), NBTError::InvalidTagName(b)) => a == b,
            (NBTError::UnexpectedType(a_type, a_tag), NBTError::UnexpectedType(b_type, b_tag)) => {
                a_type == b_type && a_tag == b_tag
            }
            (NBTError::InconsistentList, NBTError::InconsistentList) => true,
            (NBTError::InvalidListElementType(a), NBTError::InvalidListElementType(b)) => a == b,
            (NBTError::InvalidPayloadType, NBTError::InvalidPayloadType) => true,

            // not sure if it's correct but probably best we can do
            (NBTError::IO(a), NBTError::IO(b)) => a.kind() == b.kind(),

            _ => false,
        }
    }
}
