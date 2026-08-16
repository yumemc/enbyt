use std::collections::HashMap;

use strum::{EnumDiscriminants, FromRepr};

use crate::error::NBTError;

/// A container data structure holding some data.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// The name of the payload. This can be [`None`] in the case of [`TagPayload::Empty`] payloads.
    pub name: Option<String>,

    /// The data of the tag.
    pub payload: TagPayload,
}

impl Tag {
    /// Creates a new [`Tag`], given an optional name and a [`TagPayload`], ensuring the given data is
    /// valid.
    ///
    /// # Errors
    ///
    /// This function may not succeed under the following cases:
    /// - a `name` is given to a [`TagPayload::Empty`] tag
    /// - no `name` is given to a non- [`TagPayload::Empty`] tag
    /// - an inconsistent [`TagPayload::Compound`] is given (i.e. a compound payload where the keys
    ///   of the entries don't match the names of the tags)
    ///
    /// # Examples
    /// ```
    /// use enbyt::{Tag, TagPayload};
    ///
    /// let thirty = Tag::new(
    ///     Some(String::from("thirty")),
    ///     TagPayload::Int(30)
    /// ).expect("couldn't create tag");
    /// ```
    pub fn new(name: Option<String>, payload: TagPayload) -> Result<Self, NBTError> {
        match (&name, &payload) {
            // reject names for Empty tags
            (Some(_), TagPayload::Empty) => Err(NBTError::InvalidTagName(name)),
            (None, TagPayload::Empty) => Ok(Self { name, payload }),

            // reject nameless non-Empty tags
            (None, _) => Err(NBTError::InvalidTagName(None)),

            // reject inconsistent Compount tags
            (_, TagPayload::Compound(_)) if !payload.is_consistent().unwrap() => {
                Err(NBTError::InconsistentCompound)
            }
            _ => Ok(Self { name, payload }),
        }
    }

    /// Returns the [`TagPayloadType`] of the payload.
    #[must_use]
    pub fn payload_type(&self) -> TagPayloadType {
        self.payload.payload_type()
    }

    /// Returns the ID (as per the spec) of the payload's type.
    ///
    /// Calls [`TagPayload::type_id`].
    #[must_use]
    pub fn type_id(&self) -> u8 {
        self.payload.type_id()
    }
}

/// The data held by a [`Tag`] container.
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(TagPayloadType), derive(FromRepr, Hash), vis(pub))]
#[repr(u8)]
pub enum TagPayload {
    /// A zero-byte payload, typically used to indicate the end of a [`TagPayload::Compound`].
    Empty = 0x00,

    /// A signed 1-byte numeric payload.
    Byte(i8) = 0x01,

    /// A signed 2-byte numeric payload.
    Short(i16) = 0x02,

    /// A signed 4-byte numeric payload.
    Int(i32) = 0x03,

    /// A signed 8-byte numeric payload.
    Long(i64) = 0x04,

    /// A 4-byte floating point numeric payload.
    Float(f32) = 0x05,

    /// A 8-byte floating point numeric payload.
    Double(f64) = 0x06,

    /// A list payload containing several signed 2-byte numbers.
    ByteArray(Vec<i8>) = 0x07,

    /// A textual payload.
    String(String) = 0x08,

    /// A list payload, containing several [`Tag`]s.
    ///
    /// Tuple containing:
    /// - [`TagPayloadType`] denoting the Type of the tags inside the list.
    /// - [`Vec<Tag>`] containing the tags.
    ///
    /// All items inside the [`Vec<Tag>`] must have a consistent type, and be of the type denoted by
    /// the first member of the tuple.
    List(TagPayloadType, Vec<Tag>) = 0x09,

    /// A collection payload containing [`Tag`]s indexed by their name.
    ///
    /// All elements of the [`HashMap<String, Tag>`] must be consistent, i.e. their keys must match
    /// the [`Tag::name`] of the [`Tag`].
    Compound(HashMap<String, Tag>) = 0x0a,

    /// A list payload containing several signed 4-byte numbers.
    IntArray(Vec<i32>) = 0x0b,

    /// A list payload containing several signed 8-byte numbers.
    LongArray(Vec<i64>) = 0x0c,
}

impl TagPayloadType {
    pub const fn id(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for TagPayloadType {
    type Error = NBTError;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        Self::from_repr(id).ok_or(NBTError::InvalidPayloadType)
    }
}

impl TagPayload {
    /// Returns the [`TagPayloadType`] of the payload.
    #[must_use]
    pub fn payload_type(&self) -> TagPayloadType {
        self.into()
    }

    /// Returns the ID (as per the spec) of the payload's type.
    #[must_use]
    pub fn type_id(&self) -> u8 {
        self.payload_type() as u8
    }

    /// Checks if the [`HashMap`]'s keys are the same as the tag names.
    ///
    /// This works only for [`TagPayload::Compound`] payloads
    pub fn is_consistent(&self) -> Option<bool> {
        match self {
            TagPayload::Compound(map) => Some(!map.iter().any(|entry| {
                // returning true here means that this is *inconsistent*
                match &entry.1.name {
                    Some(name) => entry.0 != name,
                    None => true,
                }
            })),
            _ => None,
        }
    }
}

impl PartialEq for TagPayload {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Byte(left), Self::Byte(right)) => left == right,
            (Self::Short(left), Self::Short(right)) => left == right,
            (Self::Int(left), Self::Int(right)) => left == right,
            (Self::Long(left), Self::Long(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left.to_bits() == right.to_bits(),
            (Self::Double(left), Self::Double(right)) => left.to_bits() == right.to_bits(),
            (Self::String(left), Self::String(right)) => left == right,
            (Self::List(left1, left2), Self::List(right1, right2)) => {
                left1 == right1 && left2 == right2
            }
            (Self::Compound(left), Self::Compound(right)) => left == right,
            (Self::ByteArray(left), Self::ByteArray(right)) => left == right,
            (Self::IntArray(left), Self::IntArray(right)) => left == right,
            (Self::LongArray(left), Self::LongArray(right)) => left == right,
            _ => false,
        }
    }
}
