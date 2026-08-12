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

    use crate::{Tag, TagPayload, binary::deserialize::parse_string};

    mod serialize {
        use byteorder::{BigEndian, ByteOrder};

        pub fn write_string(buf: &mut [u8], str: String) {
            let length = str.len();
            let string_bytes = str.as_bytes();

            BigEndian::write_u16(buf, length as u16);
            buf[2..2 + string_bytes.len()].copy_from_slice(string_bytes);
        }

        #[cfg(test)]
        mod tests {
            use hegel::TestCase;
            use hegel::generators as gs;

            use crate::binary::serialize::*;

            #[hegel::test]
            fn test_write_string_length(tc: TestCase) {
                let str = tc.draw(gs::text());

                let mut buf = vec![0; 2 + str.len()];
                write_string(&mut buf, str.clone());

                let len_bytes = &buf[..2];
                let len = BigEndian::read_u16(len_bytes) as usize;

                assert_eq!(len, str.len());
            }

            #[hegel::test]
            fn test_write_string_value(tc: TestCase) {
                let str = tc.draw(gs::text());

                let mut buf = vec![0; 2 + str.len()];
                write_string(&mut buf, str.clone());

                let str_bytes = &buf[2..];
                let str_decoded = String::from_utf8(str_bytes.into());

                assert_eq!(str_decoded, Ok(str));
            }
        }
    }

    mod deserialize {

        use byteorder::{BigEndian, ByteOrder};
        use winnow::{
            ModalResult, Parser,
            error::{ContextError, StrContext},
            token::take,
        };

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

        #[cfg(test)]
        mod tests {
            use hegel::TestCase;
            use hegel::generators as gs;

            use crate::binary::deserialize::parse_string;
            use crate::binary::serialize::write_string;

            #[hegel::test]
            fn test_parse_string(tc: TestCase) {
                let str = tc.draw(gs::text());

                let mut buf = vec![0; 2 + str.len()];
                write_string(&mut buf, str.clone());

                assert_eq!(parse_string(&mut &buf[..]), Ok(str));
            }
        }
    }

    fn parse_tag_name(input: &mut &[u8]) -> ModalResult<String> {
        parse_string.parse_next(input)
    }

    fn parse_byte_payload(input: &mut &[u8]) -> ModalResult<i8> {
        take(1usize).parse_next(input).map(|bytes| {
            let byte = *bytes.first().unwrap();

            byte as i8
        })
    }

    fn parse_short_payload(input: &mut &[u8]) -> ModalResult<i16> {
        take(2usize).parse_next(input).map(BigEndian::read_i16)
    }

    fn parse_int_payload(input: &mut &[u8]) -> ModalResult<i32> {
        take(4usize).parse_next(input).map(BigEndian::read_i32)
    }

    fn parse_long_payload(input: &mut &[u8]) -> ModalResult<i64> {
        take(8usize).parse_next(input).map(BigEndian::read_i64)
    }

    fn parse_float_payload(input: &mut &[u8]) -> ModalResult<f32> {
        take(4usize).parse_next(input).map(BigEndian::read_f32)
    }

    fn parse_double_payload(input: &mut &[u8]) -> ModalResult<f64> {
        take(8usize).parse_next(input).map(BigEndian::read_f64)
    }

    fn parse_byte_array_payload(input: &mut &[u8]) -> ModalResult<Vec<u8>> {
        let len = take(4usize).parse_next(input).map(BigEndian::read_i32)? as usize;

        // TODO: zero copy?
        let bytes = take(len).parse_next(input)?;

        Ok(bytes.into())
    }

    fn parse_string_payload(input: &mut &[u8]) -> ModalResult<String> {
        parse_string.parse_next(input)
    }

    fn parse_list_payload(input: &mut &[u8]) -> ModalResult<(i8, Vec<Tag>)> {
        let tag_type_id = any.parse_next(input)? as i8;
        let size = take(4usize).parse_next(input).map(BigEndian::read_i32)? as usize;

        let tags = repeat(size, parse_tag).parse_next(input)?;

        Ok((tag_type_id, tags))
    }

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

    #[cfg(test)]
    mod tests {
        use byteorder::{BigEndian, ByteOrder};
        use hegel::TestCase;
        use hegel::generators as gs;

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

        #[test]
        fn test_empty_tag() {
            let mut input: &[u8] = &[0x00u8];

            assert_eq!(
                parse_tag(&mut input),
                Ok(Tag::new(String::default(), TagPayload::Empty))
            );
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
