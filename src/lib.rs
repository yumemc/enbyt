use std::collections::HashMap;

use winnow::{combinator, prelude::*, token};

/// enbyt: a Rust NBT library
///
/// NBT Format Reference: https://minecraft.wiki/w/NBT_format

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub payload: TagPayload,
}

#[derive(Debug)]
pub enum TagPayload {
    Byte(i8),
    Boolean(bool),
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

fn parse_tag_type_id(input: &mut &[u8]) -> ModalResult<u8> {
    token::one_of(0x00..=0x00C).parse_next(input)
}

fn parse_nbt_tag(input: &mut &[u8]) -> ModalResult<Tag> {
    let type_id = parse_tag_type_id(input)?;

    match type_id {
        0x00 => todo!("end"),
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
        _ => combinator::fail,
    }
}
