use std::{collections::HashMap, io::Read};

use flate2::read::GzDecoder;
use winnow::{
    ModalResult, Parser,
    binary::{be_f32, be_f64, be_i16, be_i32, be_i64, be_u16},
    combinator::{dispatch, empty, fail, repeat, seq},
    error::{ContextError, StrContext},
    token::{any, take},
};

use crate::{NBTError, Tag, TagPayload, TagPayloadType};

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

/// Parses an NBT array tag's payload from a byte slice `input` into a tuple containing the type of
/// the payloads and the vec of payloads.
pub fn parse_list_payload(input: &mut &[u8]) -> ModalResult<(TagPayloadType, Vec<TagPayload>)> {
    let tag_type_id = any.parse_next(input)? as i8;
    let tag_type: TagPayloadType = (tag_type_id as u8)
        .try_into()
        .map_err(|_| winnow::error::ErrMode::Cut(ContextError::new()))?;

    let size = be_i32.parse_next(input)? as usize;

    let tags = repeat(size, parse_payload(tag_type)).parse_next(input)?;

    Ok((tag_type, tags))
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

/// Parses a [`TagPayload`] of a given [`TagPayloadType`] `ty` from a byte slice `input`.
///
/// Routes to one of the below methods:
///
/// | Type                         | Parser                       |
/// | -----------------------------|------------------------------|
/// | [`TagPayloadType::Byte`]     | [`parse_byte_payload`]       |
/// | [`TagPayloadType::Short`]    | [`parse_short_payload`]      |
/// | [`TagPayloadType::Int`]      | [`parse_int_payload`]        |
/// | [`TagPayloadType::Long`]     | [`parse_long_payload`]       |
/// | [`TagPayloadType::Float`]    | [`parse_float_payload`]      |
/// | [`TagPayloadType::Double`]   | [`parse_double_payload`]     |
/// | [`TagPayloadType::Byte`]     | [`parse_byte_array_payload`] |
/// | [`TagPayloadType::String`]   | [`parse_string_payload`]     |
/// | [`TagPayloadType::Compound`] | [`parse_compound_payload`]   |
/// | [`TagPayloadType::Int`]      | [`parse_int_array_payload`]  |
/// | [`TagPayloadType::Long`]     | [`parse_long_array_payload`] |
pub fn parse_payload(ty: TagPayloadType) -> impl FnMut(&mut &[u8]) -> ModalResult<TagPayload> {
    move |input: &mut &[u8]| match ty {
        TagPayloadType::Empty => Ok(TagPayload::Empty),
        TagPayloadType::Byte => parse_byte_payload.map(TagPayload::Byte).parse_next(input),
        TagPayloadType::Short => parse_short_payload.map(TagPayload::Short).parse_next(input),
        TagPayloadType::Int => parse_int_payload.map(TagPayload::Int).parse_next(input),
        TagPayloadType::Long => parse_long_payload.map(TagPayload::Long).parse_next(input),
        TagPayloadType::Float => parse_float_payload.map(TagPayload::Float).parse_next(input),
        TagPayloadType::Double => parse_double_payload
            .map(TagPayload::Double)
            .parse_next(input),
        TagPayloadType::ByteArray => parse_byte_array_payload
            .map(TagPayload::ByteArray)
            .parse_next(input),
        TagPayloadType::String => parse_string_payload
            .map(TagPayload::String)
            .parse_next(input),
        TagPayloadType::List => parse_list_payload
            .map(|(ty, list)| TagPayload::List(ty, list))
            .parse_next(input),
        TagPayloadType::Compound => parse_compound_payload
            .map(TagPayload::Compound)
            .parse_next(input),
        TagPayloadType::IntArray => parse_int_array_payload
            .map(TagPayload::IntArray)
            .parse_next(input),
        TagPayloadType::LongArray => parse_long_array_payload
            .map(TagPayload::LongArray)
            .parse_next(input),
    }
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

/// Parses a gzip-compressed NBT tag from a reader implementing [`Read`].
///
/// Often, NBT data from Minecraft is compressed.
pub fn parse_compressed_tag<R: Read>(r: R) -> Result<Tag, NBTError> {
    let mut decoder = GzDecoder::new(r);

    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;

    let tag = parse_tag(&mut &buf[..]).map_err(NBTError::ParsingError)?;

    Ok(tag)
}
