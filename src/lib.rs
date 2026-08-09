use std::collections::HashMap;

use byteorder::{BigEndian, ByteOrder};
use winnow::{
    combinator::{self, dispatch, empty, fail, todo},
    error::{ContextError, InputError, ParserError, StrContext},
    prelude::*,
    token::{self, any, take},
};

/// enbyt: a Rust NBT library
///
/// NBT Format Reference: https://minecraft.wiki/w/NBT_format

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
    List(Vec<Tag>),

    // NOTE: Must hold that key == value.name
    Compound(HashMap<String, Tag>),

    ByteArray(Vec<u8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

fn parse_string(input: &mut &[u8]) -> ModalResult<String> {
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

fn parse_nbt_tag_name(input: &mut &[u8]) -> ModalResult<String> {
    parse_string.parse_next(input)
}

fn parse_nbt_non_empty_tag(type_id: u8) -> impl FnMut(&mut &[u8]) -> ModalResult<Tag> {
    move |input: &mut &[u8]| {
        let tag_name = parse_nbt_tag_name
            .context(StrContext::Label("NBT tag name"))
            .parse_next(input);

        match type_id {
            0x01 => todo!("byte"),
            0x02 => todo!("short"),
            0x03 => todo!("int"),
            0x04 => todo!("long"),
            0x05 => todo!("float"),
            0x06 => todo!("double"),
            0x07 => todo!("byte array"),
            0x08 => todo!("string"),
            0x09 => todo!("list"),
            0x0a => todo!("compound"),
            0x0b => todo!("int array"),
            0x0c => todo!("long array"),
            // _ => |_: &mut &[u8]| Err(winnow::error::ErrMode::Cut(ContextError::new())),
            _ => fail::<_, _, _>.parse_next(input),
        }
    }
}

pub fn parse_nbt_tag(input: &mut &[u8]) -> ModalResult<Tag> {
    dispatch! { any; // <-- TODO: can't this just be take 1 byte?
        0x0 => empty.value(Tag::new(String::default(), TagPayload::Empty)),
        type_id => parse_nbt_non_empty_tag(type_id),
    }
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, ByteOrder};

    use crate::{Tag, TagPayload, parse_nbt_tag, parse_string};

    #[test]
    fn test_empty_tag() {
        let mut input: &[u8] = &[0x00u8];

        assert_eq!(
            parse_nbt_tag(&mut input),
            Ok(Tag::new(String::default(), TagPayload::Empty))
        );
    }

    #[test]
    fn test_parse_string() {
        let string = "carly".to_string();
        let string_utf8 = string.as_bytes();

        let mut input: Vec<u8> = vec![0, 0];
        BigEndian::write_u16(input.as_mut_slice(), string_utf8.len() as u16);
        input.extend_from_slice(string_utf8);

        assert_eq!(parse_string(&mut &input[..]), Ok(string));
    }
}
