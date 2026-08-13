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

    use byteorder::{BigEndian, ByteOrder};
    use winnow::{
        combinator::{dispatch, empty, fail, repeat, seq},
        error::{ContextError, StrContext},
        prelude::*,
        token::{any, take},
    };

    use crate::{
        Tag, TagPayload,
        binary::deserialize::{
            parse_byte_array_payload, parse_byte_payload, parse_double_payload,
            parse_float_payload, parse_int_payload, parse_list_payload, parse_long_payload,
            parse_short_payload, parse_string, parse_string_payload, parse_tag_name,
        },
    };

    pub mod serialize {
        use byteorder::{BigEndian, ByteOrder};

        /// Writes a string `str` into a buffer `buf`.
        ///
        /// The format is: 16 byte integer (Big Endian) indicating the string's length, and the
        /// string's bytes encoded using UTF-8.
        pub fn write_string(buf: &mut [u8], str: String) {
            let length = str.len();
            let string_bytes = str.as_bytes();

            BigEndian::write_u16(buf, length as u16);
            buf[2..2 + string_bytes.len()].copy_from_slice(string_bytes);
        }

        /// Writes a tag name `name` into a buffer `buf`.
        ///
        /// This wraps [`write_string`].
        pub fn write_tag_name(buf: &mut [u8], name: String) {
            write_string(buf, name)
        }

        /// Writes a a signed 1 byte number `byte` into a buffer `buf`.
        ///
        /// This encodes it in two's complement form.
        pub fn write_byte_payload(buf: &mut [u8], byte: i8) {
            buf[0] = byte as u8;
        }

        /// Writes a a signed 2 byte number `value` into a buffer `buf`.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_short_payload(buf: &mut [u8], value: i16) {
            BigEndian::write_i16(buf, value);
        }

        /// Writes a a signed 4 byte number `value` into a buffer `buf`.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_int_payload(buf: &mut [u8], value: i32) {
            BigEndian::write_i32(buf, value);
        }

        /// Writes a a signed 8 byte number `value` into a buffer `buf`.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_long_payload(buf: &mut [u8], value: i64) {
            BigEndian::write_i64(buf, value);
        }

        /// Writes a a signed 4 byte float number `value` into a buffer `buf`.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_float_payload(buf: &mut [u8], value: f32) {
            BigEndian::write_f32(buf, value);
        }

        /// Writes a a signed 8 byte float number `value` into a buffer `buf`.
        ///
        /// This encodes it in Big Endian form.
        pub fn write_double_payload(buf: &mut [u8], value: f64) {
            BigEndian::write_f64(buf, value);
        }

        /// Writes a NBT byte array `arr` into a buffer `buf`.
        ///
        /// The format is: 4 bytes for the size of the array (Big Endian-encoded) then the literal
        /// byte array.
        pub fn write_byte_array_payload(buf: &mut [u8], arr: &[u8]) {
            BigEndian::write_i32(buf, arr.len() as i32);

            buf[4..4 + arr.len()].copy_from_slice(arr);
        }

        /// Writes a NBT string payload `str` into a buffer `buf`.
        ///
        /// This wraps [`write_string`]
        pub fn write_string_payload(buf: &mut [u8], str: String) {
            write_string(buf, str)
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
        // 0x0b => todo!("int array"),  
        // 0x0c => todo!("long array"), 
        _ => fail::<_,_,_>
        // type_id => parse_nbt_non_empty_tag(type_id),
    }
    .parse_next(input)
        }
    }

    #[cfg(test)]
    mod tests {
        use byteorder::{BigEndian, ByteOrder};
        use hegel::TestCase;
        use hegel::generators as gs;

        use crate::binary::deserialize::parse_tag;
        use crate::binary::*;

        fn append_nbt_string(buf: &mut Vec<u8>, str: String) {
            let str_bytes = str.as_bytes();

            append_number::<_, 2>(buf, str_bytes.len() as u16, BigEndian::write_u16);
            buf.extend_from_slice(str_bytes);
        }

        fn append_number<N, const B: usize>(buf: &mut Vec<u8>, num: N, write_fn: fn(&mut [u8], N)) {
            let mut buf2 = [0x00u8; B];
            write_fn(&mut buf2, num);

            buf.extend_from_slice(&buf2[..]);
        }

        #[hegel::test]
        fn test_parse_byte_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x01];

            let tag_name = tc.draw(gs::text());
            let byte = tc.draw(gs::integers::<i8>());

            append_nbt_string(&mut input, tag_name.clone());
            input.push(byte as u8);

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::Byte(byte)))
            );
        }

        #[hegel::test]
        fn test_parse_short_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x02];

            let tag_name = tc.draw(gs::text());
            let num = tc.draw(gs::integers::<i16>());

            append_nbt_string(&mut input, tag_name.clone());
            append_number::<_, 2>(&mut input, num, BigEndian::write_i16);

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::Short(num)))
            );
        }

        #[hegel::test]
        fn test_parse_int_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x03];

            let tag_name = tc.draw(gs::text());
            let num = tc.draw(gs::integers::<i32>());

            append_nbt_string(&mut input, tag_name.clone());
            append_number::<_, 4>(&mut input, num, BigEndian::write_i32);

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::Int(num)))
            );
        }

        #[hegel::test]
        fn test_parse_long_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x04];

            let tag_name = tc.draw(gs::text());
            let num = tc.draw(gs::integers::<i64>());

            append_nbt_string(&mut input, tag_name.clone());
            append_number::<_, 8>(&mut input, num, BigEndian::write_i64);

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::Long(num)))
            );
        }

        #[hegel::test]
        fn test_parse_float_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x05];

            let tag_name = tc.draw(gs::text());
            let num = tc.draw(gs::floats::<f32>());

            append_nbt_string(&mut input, tag_name.clone());
            append_number::<_, 4>(&mut input, num, BigEndian::write_f32);

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::Float(num)))
            );
        }

        #[hegel::test]
        fn test_parse_double_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x06];

            let tag_name = tc.draw(gs::text());
            let num = tc.draw(gs::floats::<f64>());

            append_nbt_string(&mut input, tag_name.clone());
            append_number::<_, 8>(&mut input, num, BigEndian::write_f64);

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::Double(num)))
            );
        }

        #[hegel::test]
        fn test_parse_byte_array_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x07];

            let tag_name = tc.draw(gs::text());
            let data = tc.draw(gs::vecs(gs::integers::<u8>()));

            append_nbt_string(&mut input, tag_name.clone());
            append_number::<i32, 4>(&mut input, data.len() as i32, BigEndian::write_i32);
            input.extend(data.iter());

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::ByteArray(data)))
            );
        }

        #[hegel::test]
        fn test_parse_string_tag(tc: TestCase) {
            let mut input: Vec<u8> = vec![0x08];

            let tag_name = tc.draw(gs::text());
            let string = tc.draw(gs::text());

            append_nbt_string(&mut input, tag_name.clone());
            append_nbt_string(&mut input, string.clone());

            assert_eq!(
                parse_tag(&mut &input[..]),
                Ok(Tag::new(tag_name, TagPayload::String(string)))
            );
        }

        // TODO: test list
        // but i think we have to start implementing serialization first, so testing can be done
        // nicely
    }
}
