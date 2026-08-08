use std::collections::HashMap;

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
