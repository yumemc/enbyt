/// enbyt: a Rust NBT library
///
/// NBT Format Reference: https://minecraft.wiki/w/NBT_format
use std::collections::HashMap;

use byteorder::{BigEndian, ByteOrder};
use winnow::{
    combinator::{dispatch, empty, fail, seq},
    error::{ContextError, StrContext},
    prelude::*,
    token::{any, take},
};

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

fn parse_nbt_byte_tag_payload(input: &mut &[u8]) -> ModalResult<i8> {
    take(1usize).parse_next(input).map(|bytes| {
        let byte = *bytes.first().unwrap();

        byte as i8
    })
}

pub fn parse_nbt_tag(input: &mut &[u8]) -> ModalResult<Tag> {
    dispatch! { any;
        0x0 => empty.value(Tag::new(String::default(), TagPayload::Empty)),
        0x01 => seq! { Tag { name: parse_nbt_tag_name, payload: parse_nbt_byte_tag_payload.map(TagPayload::Byte) } },
        // 0x02 => todo!("short"),      
        // 0x03 => todo!("int"),        
        // 0x04 => todo!("long"),       
        // 0x05 => todo!("float"),      
        // 0x06 => todo!("double"),     
        // 0x07 => todo!("byte array"), 
        // 0x08 => todo!("string"),     
        // 0x09 => todo!("list"),       
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

    use crate::{Tag, TagPayload, parse_nbt_tag, parse_string};

    fn write_nbt_string(buf: &mut Vec<u8>, str: String) {
        let str_bytes = str.as_bytes();

        // write length to a buffer in big endian
        let mut len_buf = [0x00u8; 2];
        BigEndian::write_u16(&mut len_buf, str_bytes.len() as u16);

        buf.extend_from_slice(&len_buf);
        buf.extend_from_slice(str_bytes);
    }

    #[test]
    fn test_empty_tag() {
        let mut input: &[u8] = &[0x00u8];

        assert_eq!(
            parse_nbt_tag(&mut input),
            Ok(Tag::new(String::default(), TagPayload::Empty))
        );
    }

    #[hegel::test]
    fn test_parse_string(tc: TestCase) {
        let string = tc.draw(gs::text());
        let mut input: Vec<u8> = vec![];

        write_nbt_string(&mut input, string.clone());

        assert_eq!(parse_string(&mut &input[..]), Ok(string));
    }

    #[hegel::test]
    fn test_parse_byte_tag(tc: TestCase) {
        let mut input: Vec<u8> = vec![0x01];

        let tag_name = tc.draw(gs::text());
        let byte = tc.draw(gs::integers::<i8>());

        write_nbt_string(&mut input, tag_name.clone());
        input.push(byte as u8);

        assert_eq!(
            parse_nbt_tag(&mut &input[..]),
            Ok(Tag::new(tag_name, TagPayload::Byte(byte)))
        );
    }
}
