//! A library for Serializing/Deserializing NBT data.
//!
//! NBT Format Reference: <https://minecraft.wiki/w/NBT_format>
use std::{collections::HashMap, io};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NBTError {
    #[error("invalid tag name {0:?}")]
    InvalidTagName(Option<String>),

    #[error("item does not match expected payload type: {0:?}")]
    UnexpectedType(u8, Tag),

    #[error("not all entries have the same key as their value's name")]
    InconsistentCompound,

    #[error("io error: {0:?}")]
    IO(#[from] io::Error),
}

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

    /// Returns the ID (as per the spec) of the payload's type.
    ///
    /// Calls [`TagPayload::type_id`].
    #[must_use]
    pub fn type_id(&self) -> u8 {
        self.payload.type_id()
    }
}

/// The data held by a [`Tag`] container.
#[derive(Debug, Clone)]
pub enum TagPayload {
    /// A zero-byte payload, typically used to indicate the end of a [`TagPayload::Compound`].
    Empty,

    /// A signed 1-byte numeric payload.
    Byte(i8),

    /// A signed 2-byte numeric payload.
    Short(i16),

    /// A signed 4-byte numeric payload.
    Int(i32),

    /// A signed 8-byte numeric payload.
    Long(i64),

    /// A 4-byte floating point numeric payload.
    Float(f32),

    /// A 8-byte floating point numeric payload.
    Double(f64),

    /// A textual payload.
    String(String),

    /// A list payload, containing several [`Tag`]s.
    ///
    /// Tuple containing:
    /// - [`i8`] denoting the Type ID of the tags inside the list.
    /// - [`Vec<Tag>`] containing the items.
    ///
    /// All items inside the [`Vec<Tag>`] must have a consistent type, and be of the type denoted by
    /// the first member of the tuple.
    List(i8, Vec<Tag>),

    /// A collection payload containing [`Tag`]s indexed by their name.
    ///
    /// All elements of the [`HashMap<String, Tag>`] must be consistent, i.e. their keys must match
    /// the [`Tag::name`] of the [`Tag`].
    Compound(HashMap<String, Tag>),

    /// A list payload containing several signed 2-byte numbers.
    ByteArray(Vec<i8>),

    /// A list payload containing several signed 4-byte numbers.
    IntArray(Vec<i32>),

    /// A list payload containing several signed 8-byte numbers.
    LongArray(Vec<i64>),
}

impl TagPayload {
    /// Returns the ID (as per the spec) of the payload's type.
    #[must_use]
    pub fn type_id(&self) -> u8 {
        match self {
            crate::TagPayload::Empty => 0x00,
            crate::TagPayload::Byte(_) => 0x01,
            crate::TagPayload::Short(_) => 0x02,
            crate::TagPayload::Int(_) => 0x03,
            crate::TagPayload::Long(_) => 0x04,
            crate::TagPayload::Float(_) => 0x05,
            crate::TagPayload::Double(_) => 0x06,
            crate::TagPayload::ByteArray(_) => 0x07,
            crate::TagPayload::String(_) => 0x08,
            crate::TagPayload::List(_, _) => 0x09,
            crate::TagPayload::Compound(_) => 0x0a,
            crate::TagPayload::IntArray(_) => 0x0b,
            crate::TagPayload::LongArray(_) => 0x0c,
        }
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

pub mod binary {

    pub mod serialize {

        use std::io::Write;

        use crate::{NBTError, Tag, TagPayload};

        /// Writes a string `str` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - 2 byte integer (Big Endian) indicating the string's length
        /// - the string's bytes encoded using UTF-8.
        pub fn write_string<W: Write>(w: &mut W, str: String) -> Result<usize, NBTError> {
            let length = str.len() as u16;
            let string_bytes = str.as_bytes();

            let mut written = 0;

            written += w.write(&length.to_be_bytes())?;
            written += w.write(string_bytes)?;

            Ok(written)
        }

        /// Writes a tag name `name` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This wraps [`write_string`].
        pub fn write_tag_name<W: Write>(w: &mut W, name: String) -> Result<usize, NBTError> {
            write_string(w, name)
        }

        /// Writes a empty NBT tag into `w`.
        ///
        /// Returns the amount of bytes written.
        pub fn write_empty_payload<W: Write>(w: &mut W) -> Result<usize, NBTError> {
            Ok(w.write(&[0x00])?)
        }

        /// Writes a a signed 1 byte number `byte` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in two's complement form.
        pub fn write_byte_payload<W: Write>(w: &mut W, byte: i8) -> Result<usize, NBTError> {
            Ok(w.write(&[byte as u8])?)
        }

        /// Writes a a signed 2 byte number `value` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_short_payload<W: Write>(w: &mut W, value: i16) -> Result<usize, NBTError> {
            Ok(w.write(&value.to_be_bytes())?)
        }

        /// Writes a a signed 4 byte number `value` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_int_payload<W: Write>(w: &mut W, value: i32) -> Result<usize, NBTError> {
            Ok(w.write(&value.to_be_bytes())?)
        }

        /// Writes a a signed 8 byte number `value` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_long_payload<W: Write>(w: &mut W, value: i64) -> Result<usize, NBTError> {
            Ok(w.write(&value.to_be_bytes())?)
        }

        /// Writes a a signed 4 byte float number `value` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_float_payload<W: Write>(w: &mut W, value: f32) -> Result<usize, NBTError> {
            Ok(w.write(&value.to_be_bytes())?)
        }

        /// Writes a a signed 8 byte float number `value` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_double_payload<W: Write>(w: &mut W, value: f64) -> Result<usize, NBTError> {
            Ok(w.write(&value.to_be_bytes())?)
        }

        /// Writes a NBT byte array `arr` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - 4 bytes for the size of the array (Big Endian-encoded)
        /// - the literal byte array.
        pub fn write_byte_array_payload<W: Write>(
            w: &mut W,
            arr: Vec<i8>,
        ) -> Result<usize, NBTError> {
            let mut written = 0;

            let length = arr.len() as i32;

            written += w.write(&length.to_be_bytes())?;

            for item in arr {
                written += write_byte_payload(w, item)?;
            }

            Ok(written)
        }

        /// Writes a NBT string payload `str` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This wraps [`write_string`]
        pub fn write_string_payload<W: Write>(w: &mut W, str: String) -> Result<usize, NBTError> {
            write_string(w, str)
        }

        /// Writes a NBT list payload `list` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - 1 byte for the ID of the type of the list's contents
        /// - 4 bytes for the length
        /// - every tag
        pub fn write_list_payload<W: Write>(
            w: &mut W,
            (type_id, list): (i8, Vec<Tag>),
        ) -> Result<usize, NBTError> {
            let mut written = 0;

            let type_id = type_id as u8;

            // ensure all items are of the same type
            let first_different = list.iter().find(|tag| tag.type_id() != type_id);

            if let Some(tag) = first_different {
                return Err(NBTError::UnexpectedType(type_id, tag.clone()));
            }

            let length = list.len() as i32;

            written += w.write(&[type_id])?;
            written += w.write(&length.to_be_bytes())?;

            for item in list {
                written += write_tag(w, item)?;
            }

            Ok(written)
        }

        /// Writes a NBT compound payload `value` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - every payload inside the list (the value of this payload)
        /// - 0x00, which is an empty NBT tag
        pub fn write_compound_payload<W: Write>(
            w: &mut W,
            value: Vec<Tag>,
        ) -> Result<usize, NBTError> {
            let mut written = 0;

            for item in value {
                written += write_tag(w, item)?;
            }

            written += write_empty_payload(w)?;

            Ok(written)
        }

        /// Writes a NBT int array `arr` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - 4 bytes for the size (as in the length, not to be confused with size in bytes)
        /// - n int payloads (see [`write_int_payload`])
        pub fn write_int_array_payload<W: Write>(
            w: &mut W,
            arr: Vec<i32>,
        ) -> Result<usize, NBTError> {
            let mut written = 0;

            let length = arr.len() as i32;

            written += w.write(&length.to_be_bytes())?;

            for item in arr {
                written += write_int_payload(w, item)?;
            }

            Ok(written)
        }

        /// Writes a NBT long array `arr` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - 4 bytes for the size (as in the length, not to be confused with size in bytes)
        /// - n long payloads (see [`write_long_payload`])
        pub fn write_long_array_payload<W: Write>(
            w: &mut W,
            arr: Vec<i64>,
        ) -> Result<usize, NBTError> {
            let mut written = 0;

            let length = arr.len() as i32;

            written += w.write(&length.to_be_bytes())?;

            for item in arr {
                written += write_long_payload(w, item)?;
            }

            Ok(written)
        }

        /// Writes a NBT tag `tag` into `w`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - 1 byte for the tag type's ID
        /// - 2 bytes for the length of the name (see [`write_string`])
        /// - n bytes for the name's UTF-8 encoded
        /// - the tag's payload
        pub fn write_tag<W: Write>(w: &mut W, tag: Tag) -> Result<usize, NBTError> {
            let mut written = 0;

            let tag_type_id = tag.type_id();

            written += w.write(&tag_type_id.to_be_bytes())?;

            written += match tag.name {
                Some(name) => write_string(w, name)?,
                None => 0,
            };

            written += match tag.payload {
                TagPayload::Empty => Ok(0),
                TagPayload::Byte(value) => write_byte_payload(w, value),
                TagPayload::Short(value) => write_short_payload(w, value),
                TagPayload::Int(value) => write_int_payload(w, value),
                TagPayload::Long(value) => write_long_payload(w, value),
                TagPayload::Float(value) => write_float_payload(w, value),
                TagPayload::Double(value) => write_double_payload(w, value),
                TagPayload::String(value) => write_string_payload(w, value),
                TagPayload::List(type_id, value) => write_list_payload(w, (type_id, value)),
                TagPayload::Compound(value) => {
                    write_compound_payload(w, value.into_values().collect())
                }
                TagPayload::ByteArray(value) => write_byte_array_payload(w, value),
                TagPayload::IntArray(items) => write_int_array_payload(w, items),
                TagPayload::LongArray(items) => write_long_array_payload(w, items),
            }?;

            Ok(written)
        }
    }

    pub mod deserialize {

        use std::collections::HashMap;

        use winnow::{
            ModalResult, Parser,
            binary::{be_f32, be_f64, be_i16, be_i32, be_i64, be_u16},
            combinator::{dispatch, empty, fail, repeat, seq},
            error::{ContextError, StrContext},
            token::{any, take},
        };

        use crate::{Tag, TagPayload};

        /// Parses a string from a byte slice `input` into a [`String`].
        pub fn parse_string(input: &mut &[u8]) -> ModalResult<String> {
            // 2 bytes of length
            let length = be_u16
                .context(StrContext::Label("string length"))
                .parse_next(input)?;

            // n bytes of string
            let string_bytes = take(length as usize)
                .context(StrContext::Label("string"))
                .parse_next(input)?;

            let string = String::from_utf8(string_bytes.to_vec())
                .map_err(|_| winnow::error::ErrMode::Cut(ContextError::new()))?; // TODO: add context :)

            Ok(string)
        }

        /// Parses a tag's name from a byte slice `input` into a [`String`].
        ///
        /// This wraps [`parse_string`].
        pub fn parse_tag_name(input: &mut &[u8]) -> ModalResult<String> {
            parse_string.parse_next(input)
        }

        /// Parses a NBT byte tag's payload from a byte slice `input` into an [`i8`].
        pub fn parse_byte_payload(input: &mut &[u8]) -> ModalResult<i8> {
            take(1usize).parse_next(input).map(|bytes| {
                let byte = *bytes.first().unwrap();

                byte as i8
            })
        }

        /// Parses a NBT short tag's payload from a byte slice `input` into an [`i16`].
        pub fn parse_short_payload(input: &mut &[u8]) -> ModalResult<i16> {
            be_i16.parse_next(input)
        }

        /// Parses a NBT int tag's payload from a byte slice `input` into an [`i32`].
        pub fn parse_int_payload(input: &mut &[u8]) -> ModalResult<i32> {
            be_i32.parse_next(input)
        }

        /// Parses a NBT long tag's payload from a byte slice `input` into an [`i64`].
        pub fn parse_long_payload(input: &mut &[u8]) -> ModalResult<i64> {
            be_i64.parse_next(input)
        }

        /// Parses a NBT float tag's payload from a byte slice `input` into an [`f32`].
        pub fn parse_float_payload(input: &mut &[u8]) -> ModalResult<f32> {
            be_f32.parse_next(input)
        }

        /// Parses a NBT float tag's payload from a byte slice `input` into an [`f64`].
        pub fn parse_double_payload(input: &mut &[u8]) -> ModalResult<f64> {
            be_f64.parse_next(input)
        }

        /// Parses a NBT byte array tag's payload from a byte slice `input` into a [`Vec<u8>`].
        pub fn parse_byte_array_payload(input: &mut &[u8]) -> ModalResult<Vec<i8>> {
            let len = be_i32.parse_next(input)? as usize;

            let mut bytes = vec![];

            // TODO: make this not imperative
            for _ in 0..len {
                bytes.push(parse_byte_payload(input)?);
            }

            Ok(bytes)
        }

        /// Parses an NBT string tag's payload from a byte slice `input` into a [`String`].
        ///
        /// This wraps [`parse_string`].
        pub fn parse_string_payload(input: &mut &[u8]) -> ModalResult<String> {
            parse_string.parse_next(input)
        }

        /// Parses an NBT array tag's payload from a byte slice `input` into a [`(i8, Vec<Tag>)`]
        /// containing the size and the tag array.
        pub fn parse_list_payload(input: &mut &[u8]) -> ModalResult<(i8, Vec<Tag>)> {
            let tag_type_id = any.parse_next(input)? as i8;
            let size = be_i32.parse_next(input)? as usize;

            let tags = repeat(size, parse_tag).parse_next(input)?;

            // TODO: as per the reference, the tags are all of the same type. Should this return an
            // error if they are not? This would risk losing corrupt data with the user unable to do
            // anything about it. Perhaps some sort of `strict` option.

            Ok((tag_type_id, tags))
        }

        /// Parses an NBT compound tag's payload from a byte slice `input` into a [`HashMap<String, Tag>`].
        pub fn parse_compound_payload(input: &mut &[u8]) -> ModalResult<HashMap<String, Tag>> {
            let mut tags_map = HashMap::new();

            loop {
                let tag = parse_tag.parse_next(input)?;

                // stop on empty payload (delimeter)
                if let TagPayload::Empty = tag.payload {
                    break;
                }

                // if there's no name, then we have no key for the output map, so we can't continue.
                //
                // (we *could* recover from this and just continue if we were to make the API return
                // a collection of tags instead, in which case we couldn't need a key, but I'm not
                // sure if that's ideal.)
                match &tag.name {
                    Some(name) => tags_map.insert(name.clone(), tag),
                    // TODO: add context
                    None => return Err(winnow::error::ErrMode::Cut(ContextError::new())),
                };
            }

            Ok(tags_map)
        }

        /// Parses a NBT int array tag's payload from a byte slice `input` into a [`Vec<i32>`].
        pub fn parse_int_array_payload(input: &mut &[u8]) -> ModalResult<Vec<i32>> {
            let len = be_i32.parse_next(input)? as usize;

            // TODO: zero copy?
            let mut ints = vec![];

            // TODO: make this not imperative
            for _ in 0..len {
                ints.push(parse_int_payload(input)?);
            }

            Ok(ints)
        }

        /// Parses a NBT long array tag's payload from a byte slice `input` into a [`Vec<i64>`].
        pub fn parse_long_array_payload(input: &mut &[u8]) -> ModalResult<Vec<i64>> {
            let len = be_i32.parse_next(input)? as usize;

            // TODO: zero copy?
            let mut ints = vec![];

            // TODO: make this not imperative
            for _ in 0..len {
                ints.push(parse_long_payload(input)?);
            }

            Ok(ints)
        }

        /// Parses an NBT tag from a byte slice `input` into a [`Tag`].
        pub fn parse_tag(input: &mut &[u8]) -> ModalResult<Tag> {
            let tag_name = || parse_tag_name.map(Some);
            let make_tag = |(name, payload)| Tag::new(name, payload);

            dispatch! { any;
                0x0 => empty.try_map(|()| Tag::new(None, TagPayload::Empty)),
                0x01 => seq!((tag_name(), parse_byte_payload.map(TagPayload::Byte))).try_map(make_tag),
                0x02 => seq!((tag_name(), parse_short_payload.map(TagPayload::Short))).try_map(make_tag),
                0x03 => seq!((tag_name(), parse_int_payload.map(TagPayload::Int))).try_map(make_tag),
                0x04 => seq!((tag_name(), parse_long_payload.map(TagPayload::Long))).try_map(make_tag),
                0x05 => seq!((tag_name(), parse_float_payload.map(TagPayload::Float))).try_map(make_tag),
                0x06 => seq!((tag_name(), parse_double_payload.map(TagPayload::Double))).try_map(make_tag),
                0x07 => seq!((tag_name(), parse_byte_array_payload.map(TagPayload::ByteArray))).try_map(make_tag),
                0x08 => seq!((tag_name(), parse_string_payload.map(TagPayload::String))).try_map(make_tag),
                0x09 => seq!((tag_name(), parse_list_payload.map(|(id, tags)| TagPayload::List(id, tags)))).try_map(make_tag),
                0x0a => seq!((tag_name(), parse_compound_payload.map(TagPayload::Compound))).try_map(make_tag),
                0x0b => seq!((tag_name(), parse_int_array_payload.map(TagPayload::IntArray))).try_map(make_tag),
                0x0c => seq!((tag_name(), parse_long_array_payload.map(TagPayload::LongArray))).try_map(make_tag),
                _ => fail::<_,_,_>
                // type_id => parse_nbt_non_empty_tag(type_id),
            }
            .parse_next(input)
        }
    }
}
