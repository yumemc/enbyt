/// enbyt: a Rust NBT library
///
/// NBT Format Reference: https://minecraft.wiki/w/NBT_format
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    pub payload: TagPayload,
}

impl Tag {
    pub fn new(name: String, payload: TagPayload) -> Self {
        Self { name, payload }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagPayload {
    Empty,

    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),

    // NOTE: Should be homogenous
    List(i8, Vec<Tag>),

    // NOTE: Must hold that key == value.name
    Compound(HashMap<String, Tag>),

    ByteArray(Vec<u8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

pub mod binary {

    pub mod serialize {
        use byteorder::{BigEndian, ByteOrder};

        use crate::{Tag, TagPayload};

        /// Writes a string `str` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is: 2 byte integer (Big Endian) indicating the string's length, and the
        /// string's bytes encoded using UTF-8.
        pub fn write_string(buf: &mut [u8], str: String) -> usize {
            let length = str.len();
            let string_bytes = str.as_bytes();

            BigEndian::write_u16(buf, length as u16);
            buf[2..2 + string_bytes.len()].copy_from_slice(string_bytes);

            2 + string_bytes.len()
        }

        /// Writes a tag name `name` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This wraps [`write_string`].
        pub fn write_tag_name(buf: &mut [u8], name: String) -> usize {
            write_string(buf, name)
        }

        /// Writes a a signed 1 byte number `byte` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in two's complement form.
        pub fn write_byte_payload(buf: &mut [u8], byte: i8) -> usize {
            buf[0] = byte as u8;

            1
        }

        /// Writes a a signed 2 byte number `value` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_short_payload(buf: &mut [u8], value: i16) -> usize {
            BigEndian::write_i16(buf, value);

            2
        }

        /// Writes a a signed 4 byte number `value` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_int_payload(buf: &mut [u8], value: i32) -> usize {
            BigEndian::write_i32(buf, value);

            4
        }

        /// Writes a a signed 8 byte number `value` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_long_payload(buf: &mut [u8], value: i64) -> usize {
            BigEndian::write_i64(buf, value);

            8
        }

        /// Writes a a signed 4 byte float number `value` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_float_payload(buf: &mut [u8], value: f32) -> usize {
            BigEndian::write_f32(buf, value);

            4
        }

        /// Writes a a signed 8 byte float number `value` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_double_payload(buf: &mut [u8], value: f64) -> usize {
            BigEndian::write_f64(buf, value);

            8
        }

        /// Writes a NBT byte array `arr` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is: 4 bytes for the size of the array (Big Endian-encoded) then the literal
        /// byte array.
        pub fn write_byte_array_payload(buf: &mut [u8], arr: &[u8]) -> usize {
            BigEndian::write_i32(buf, arr.len() as i32);

            buf[4..4 + arr.len()].copy_from_slice(arr);

            4 + arr.len()
        }

        /// Writes a NBT string payload `str` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// This wraps [`write_string`]
        pub fn write_string_payload(buf: &mut [u8], str: String) -> usize {
            write_string(buf, str)
        }

        /// Writes a NBT list payload `list` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        ///
        ///     - 1 byte for the ID of the type of the list's contents
        ///     - 4 bytes for the length
        ///     - every tag
        pub fn write_list_payload(buf: &mut [u8], (type_id, list): (i8, Vec<Tag>)) -> usize {
            // TODO: validate as per spec? (all tags must have the same type?)

            buf[0] = type_id as u8;

            BigEndian::write_i32(buf, list.len() as i32);

            let tags_buf = &mut buf[5..];

            let tags_written = list.iter().fold(0, |start, tag| {
                // TODO: remove clone, consider taking reference in write_tag.
                let written = write_tag(&mut tags_buf[start..], tag.clone());

                start + written
            });

            5 + tags_written
        }

        /// Writes a NBT int array `arr` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        /// - 4 bytes for the size (as in the length, not to be confused with size in bytes)
        /// - n int payloads (see [`write_int_payload`])
        pub fn write_int_array_payload(buf: &mut [u8], arr: Vec<i32>) -> usize {
            BigEndian::write_i32(buf, arr.len() as i32);

            let ints_buf = &mut buf[4..];

            let ints_written = arr.iter().fold(0, |start, val| {
                let written = write_int_payload(&mut ints_buf[start..], *val);

                start + written
            });

            4 + ints_written
        }

        /// Writes a NBT tag `tag` into a buffer `buf`.
        ///
        /// Returns the amount of bytes written.
        ///
        /// The format is:
        ///     - 1 byte for the tag type's ID
        ///
        ///     (see [`write_string`])
        ///     - 2 bytes for the length of the name
        ///     - n bytes for the name's UTF-8 encoded
        ///
        ///     - the tag's payload
        pub fn write_tag(buf: &mut [u8], tag: Tag) -> usize {
            // TODO: this could be extracted out to TagPayload::type_id()
            let tag_type_id = match tag.payload {
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
            };

            buf[0] = tag_type_id as u8;

            let name_written = write_string(&mut buf[1..], tag.name);

            let payload_buf = &mut buf[name_written..];

            let payload_written = match tag.payload {
                TagPayload::Empty => 0,
                TagPayload::Byte(value) => write_byte_payload(payload_buf, value),
                TagPayload::Short(value) => write_short_payload(payload_buf, value),
                TagPayload::Int(value) => write_int_payload(payload_buf, value),
                TagPayload::Long(value) => write_long_payload(payload_buf, value),
                TagPayload::Float(value) => write_float_payload(payload_buf, value),
                TagPayload::Double(value) => write_double_payload(payload_buf, value),
                TagPayload::String(value) => write_string_payload(payload_buf, value),
                TagPayload::List(type_id, value) => {
                    write_list_payload(payload_buf, (type_id, value))
                }
                TagPayload::Compound(hash_map) => todo!(),
                TagPayload::ByteArray(value) => write_byte_array_payload(payload_buf, &value),
                TagPayload::IntArray(items) => write_int_array_payload(payload_buf, items),
                TagPayload::LongArray(items) => todo!(),
            };

            1 + name_written + payload_written
        }
    }

    pub mod deserialize {

        use byteorder::{BigEndian, ByteOrder};
        use winnow::{
            ModalResult, Parser,
            combinator::{dispatch, empty, fail, repeat, seq},
            error::{ContextError, StrContext},
            token::{any, take},
        };

        use crate::{Tag, TagPayload};

        /// Parses a string from a byte slice `input` into a [`String`].
        pub fn parse_string(input: &mut &[u8]) -> ModalResult<String> {
            // 2 bytes of length
            let length = take(2usize)
                .context(StrContext::Label("string length"))
                .parse_next(input)
                .map(BigEndian::read_u16)?;

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
            take(2usize).parse_next(input).map(BigEndian::read_i16)
        }

        /// Parses a NBT int tag's payload from a byte slice `input` into an [`i32`].
        pub fn parse_int_payload(input: &mut &[u8]) -> ModalResult<i32> {
            take(4usize).parse_next(input).map(BigEndian::read_i32)
        }

        /// Parses a NBT long tag's payload from a byte slice `input` into an [`i64`].
        pub fn parse_long_payload(input: &mut &[u8]) -> ModalResult<i64> {
            take(8usize).parse_next(input).map(BigEndian::read_i64)
        }

        /// Parses a NBT float tag's payload from a byte slice `input` into an [`f32`].
        pub fn parse_float_payload(input: &mut &[u8]) -> ModalResult<f32> {
            take(4usize).parse_next(input).map(BigEndian::read_f32)
        }

        /// Parses a NBT float tag's payload from a byte slice `input` into an [`f64`].
        pub fn parse_double_payload(input: &mut &[u8]) -> ModalResult<f64> {
            take(8usize).parse_next(input).map(BigEndian::read_f64)
        }

        /// Parses a NBT byte array tag's payload from a byte slice `input` into a [`Vec<u8>`].
        pub fn parse_byte_array_payload(input: &mut &[u8]) -> ModalResult<Vec<u8>> {
            let len = take(4usize).parse_next(input).map(BigEndian::read_i32)? as usize;

            // TODO: zero copy?
            let bytes = take(len).parse_next(input)?;

            Ok(bytes.into())
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
            let size = take(4usize).parse_next(input).map(BigEndian::read_i32)? as usize;

            let tags = repeat(size, parse_tag).parse_next(input)?;

            // TODO: as per the reference, the tags are all of the same type. Should this return an
            // error if they are not? This would risk losing corrupt data with the user unable to do
            // anything about it. Perhaps some sort of `strict` option.

            Ok((tag_type_id, tags))
        }

        /// Parses a NBT int array tag's payload from a byte slice `input` into a [`Vec<i32>`].
        pub fn parse_int_array_payload(input: &mut &[u8]) -> ModalResult<Vec<i32>> {
            let len = take(4usize).parse_next(input).map(BigEndian::read_i32)? as usize;

            // TODO: zero copy?
            let mut ints = vec![];

            // TODO: make this not imperative
            for _ in 0..len {
                ints.push(parse_int_payload(input)?);
            }

            Ok(ints)
        }

        /// Parses an NBT tag from a byte slice `input` into a [`Tag`].
        pub fn parse_tag(input: &mut &[u8]) -> ModalResult<Tag> {
            dispatch! { any;
        0x0 => empty.value(Tag::new(String::default(), TagPayload::Empty)),
        0x01 => seq! { Tag { name: parse_tag_name, payload: parse_byte_payload.map(TagPayload::Byte) } },
        0x02 => seq! { Tag { name: parse_tag_name, payload: parse_short_payload.map(TagPayload::Short) } },
        0x03 => seq! { Tag { name: parse_tag_name, payload: parse_int_payload.map(TagPayload::Int) } },
        0x04 => seq! { Tag { name: parse_tag_name, payload: parse_long_payload.map(TagPayload::Long) } },
        0x05 => seq! { Tag { name: parse_tag_name, payload: parse_float_payload.map(TagPayload::Float) } },
        0x06 => seq! { Tag { name: parse_tag_name, payload: parse_double_payload.map(TagPayload::Double) } },
        0x07 => seq! { Tag { name: parse_tag_name, payload: parse_byte_array_payload.map(TagPayload::ByteArray) } },
        0x08 => seq! { Tag { name: parse_tag_name, payload: parse_string_payload.map(TagPayload::String) } },
        0x09 => seq! { Tag { name: parse_tag_name, payload: parse_list_payload.map(|(id, tags)| TagPayload::List(id, tags)) } },
        // 0x0a => todo!("compound"),   
        0x0b => seq! { Tag { name: parse_tag_name, payload: parse_int_array_payload.map(TagPayload::IntArray) } },
        // 0x0c => todo!("long array"), 
        _ => fail::<_,_,_>
        // type_id => parse_nbt_non_empty_tag(type_id),
    }
    .parse_next(input)
        }
    }
}
