use std::collections::HashMap;

use strum::{EnumDiscriminants, EnumIs, EnumTryAs, FromRepr};

use crate::error::NBTError;

/// A container data structure holding some data.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// The name of the payload.
    pub name: String,

    /// The data of the tag.
    pub payload: TagPayload,
}

impl Tag {
    /// Creates a new [`Tag`], given an optional name and a [`TagPayload`], ensuring the given data is
    /// valid.
    ///
    /// # Errors
    /// - a [`TagPayload::List`] is given whose elements are not all of the same type, which should
    ///   is the type (specifically [`TagPayload::List::0`]).
    ///
    /// # Examples
    /// ```
    /// use enbyt::{Tag, TagPayload};
    ///
    /// let thirty = Tag::new(
    ///     "thirty".to_string(), TagPayload::Int(30)
    /// ).expect("couldn't create tag");
    /// ```
    pub fn new(name: String, payload: TagPayload) -> Result<Self, NBTError> {
        match (&name, &payload) {
            // reject inconsistent List tags
            (_, TagPayload::List(_, _)) if !payload.is_consistent().unwrap() => {
                Err(NBTError::InconsistentList)
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

impl TryFrom<(String, TagPayload)> for Tag {
    type Error = NBTError;

    fn try_from((name, payload): (String, TagPayload)) -> Result<Self, Self::Error> {
        Tag::new(name, payload)
    }
}

impl TryFrom<(&String, &TagPayload)> for Tag {
    type Error = NBTError;

    fn try_from((name, payload): (&String, &TagPayload)) -> Result<Self, Self::Error> {
        Tag::new(name.clone(), payload.clone())
    }
}

impl From<Tag> for (String, TagPayload) {
    fn from(tag: Tag) -> Self {
        (tag.name, tag.payload)
    }
}

/// The data held by a [`Tag`] container.
#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, EnumIs)]
#[strum_discriminants(name(TagPayloadType), derive(FromRepr, Hash), vis(pub))]
#[repr(u8)]
pub enum TagPayload {
    /// End
    End = 0x00,

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

    /// A list payload, containing several [`TagPayload`]s of the same type.
    ///
    /// Tuple containing:
    /// - [`TagPayloadType`] denoting the Type of the payloads inside the list.
    /// - [`Vec<TagPayload>`] containing the payloads.
    List(TagPayloadType, Vec<TagPayload>) = 0x09,

    /// A collection payload containing [`TagPayload`]s indexed by their name.
    Compound(HashMap<String, TagPayload>) = 0x0a,

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
        Self::from_repr(id).ok_or(NBTError::InvalidPayloadType(id))
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

    /// Checks if the payload is consistent.
    ///
    /// This has a different definition depending on the type of payload:
    /// - For [`TagPayload::List`]: checks if all the elements are of the same type (which must be
    ///   the same type specified in the first part of the tuple)
    /// - For any other payload type [`None`] is returned.
    ///
    /// # Examples
    ///
    ////// ## List Tags
    /// ```
    /// use enbyt::{Tag, TagPayload, TagPayloadType};
    ///
    /// let consistent = TagPayload::List(TagPayloadType::Byte, vec![
    ///     TagPayload::Byte(0x00),
    ///     TagPayload::Byte(0x03)
    /// ]);
    ///
    /// assert_eq!(consistent.is_consistent(), Some(true));
    ///
    /// let inconsistent = TagPayload::List(TagPayloadType::Byte, vec![
    ///     TagPayload::Byte(0x00),
    ///     TagPayload::Int(0x03)
    /// ]);
    ///
    /// assert_eq!(inconsistent.is_consistent(), Some(false));
    /// ```
    pub fn is_consistent(&self) -> Option<bool> {
        match self {
            TagPayload::List(ty, list) => Some(!list.iter().any(|item| item.type_id() != ty.id())),
            _ => None,
        }
    }
}

impl PartialEq for TagPayload {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
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
