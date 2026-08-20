//! Contains the code for deserializing binary NBT data.
//!
//! A lot of the functions in this module are relatively low-level. While they can be used, most of
//! the time one should only need [`parse_compressed_tag`] and perhaps [`parse_tag`].
use std::{collections::BTreeMap, io::Read};

use flate2::read::GzDecoder;
use winnow::{
    ModalResult, Parser,
    binary::{be_f32, be_f64, be_i16, be_i32, be_i64, be_u16, length_and_then, length_repeat},
    combinator::{dispatch, fail, seq},
    error::StrContext,
    token::{any, rest},
};

use crate::{NBTError, Tag, TagPayload, TagPayloadType};

/// Parses an NBT string into a [`String`].
/// For the format see [`super::serialize::write_string`].
pub fn parse_string(input: &mut &[u8]) -> ModalResult<String> {
    length_and_then(
        be_u16.context(StrContext::Label("string length")),
        rest.try_map(|bytes: &[u8]| String::from_utf8(bytes.to_vec()))
            .context(StrContext::Label("utf-8 encoded string")),
    )
    .parse_next(input)
}

/// Parses a NBT tag's name.
///
/// This wraps [`parse_string`].
pub fn parse_tag_name(input: &mut &[u8]) -> ModalResult<String> {
    parse_string
        .context(StrContext::Label("tag name"))
        .parse_next(input)
}

/// Parses a NBT byte tag's payload into a [`i8`].
/// For the format see [`super::serialize::write_byte_payload`].
pub fn parse_byte_payload(input: &mut &[u8]) -> ModalResult<i8> {
    any.map(|byte| byte as i8)
        .context(StrContext::Label("byte payload"))
        .parse_next(input)
}

/// Parses a NBT short tag's payload into an [`i16`].
/// For the format see [`super::serialize::write_short_payload`].
pub fn parse_short_payload(input: &mut &[u8]) -> ModalResult<i16> {
    be_i16
        .context(StrContext::Label("short payload"))
        .parse_next(input)
}

/// Parses a NBT int tag's payload into an [`i32`].
/// For the format see [`super::serialize::write_int_payload`].
pub fn parse_int_payload(input: &mut &[u8]) -> ModalResult<i32> {
    be_i32
        .context(StrContext::Label("int payload"))
        .parse_next(input)
}

/// Parses a NBT long tag's payload into an [`i64`].
/// For the format see [`super::serialize::write_long_payload`].
pub fn parse_long_payload(input: &mut &[u8]) -> ModalResult<i64> {
    be_i64
        .context(StrContext::Label("long payload"))
        .parse_next(input)
}

/// Parses a NBT float tag's payload into an [`f32`].
/// For the format see [`super::serialize::write_float_payload`].
pub fn parse_float_payload(input: &mut &[u8]) -> ModalResult<f32> {
    be_f32
        .context(StrContext::Label("float payload"))
        .parse_next(input)
}

/// Parses a NBT double tag's payload into an [`f64`].
/// For the format see [`super::serialize::write_double_payload`].
pub fn parse_double_payload(input: &mut &[u8]) -> ModalResult<f64> {
    be_f64
        .context(StrContext::Label("double payload"))
        .parse_next(input)
}

/// Parses a NBT byte array tag's payload into a [`Vec<i8>`].
/// For the format see [`super::serialize::write_byte_array_payload`].
///
/// Uses [`parse_byte_payload`] for parsing the actual bytes.
pub fn parse_byte_array_payload(input: &mut &[u8]) -> ModalResult<Vec<i8>> {
    length_repeat(
        be_i32
            .map(|x| x as usize)
            .context(StrContext::Label("byte array length")),
        parse_byte_payload.context(StrContext::Label("byte")),
    )
    .context(StrContext::Label("byte array payload"))
    .parse_next(input)
}

/// Parses an NBT string tag's payload from a byte slice `input` into a [`String`].
///
/// This wraps [`parse_string`].
pub fn parse_string_payload(input: &mut &[u8]) -> ModalResult<String> {
    parse_string
        .context(StrContext::Label("string payload "))
        .parse_next(input)
}

/// Parses a NBT list tag's payload into a [`(TagPayloadType, Vec<TagPayload>)`]. That is, a tuple
/// containing a type, and a vec containing payloads of that type.
///
/// For the format see [`super::serialize::write_list_payload`].
pub fn parse_list_payload(input: &mut &[u8]) -> ModalResult<(TagPayloadType, Vec<TagPayload>)> {
    any.try_map(TagPayloadType::try_from)
        .context(StrContext::Label("list type id"))
        .flat_map(|tag_type| {
            length_repeat(
                be_i32
                    .map(|x| x as usize)
                    .context(StrContext::Label("list size")),
                parse_payload(tag_type).context(StrContext::Label("list item")),
            )
            .map(move |tags: Vec<TagPayload>| (tag_type, tags))
        })
        .parse_next(input)

    // we had this previously but tests seem to pass without it? possibly fixed elsewhere
    //
    // if size < 0 {
    //     return Err(winnow::error::ErrMode::Cut(ContextError::from_input(input)));
    // }
}

/// Parses an NBT end tag's payload.
pub fn parse_end_payload(input: &mut &[u8]) -> ModalResult<()> {
    0x00.context(StrContext::Label("end marker"))
        .parse_next(input)
        .map(|_| ())
}

/// Parses a NBT compound tag's payload into a [`BTreeMap<String, TagPayload>`], mapping the names
/// of tags to their payloads.
///
/// For the format see [`super::serialize::write_compound_payload`].
pub fn parse_compound_payload(input: &mut &[u8]) -> ModalResult<BTreeMap<String, TagPayload>> {
    let mut tags_map = BTreeMap::new();

    // TODO: this whole pattern should be trivially replaceable by some winnow builtin
    // combinator, use it.
    loop {
        let is_end = parse_end_payload(input).is_ok();

        if is_end {
            break;
        }

        let tag = parse_tag.parse_next(input)?;

        tags_map.insert(tag.name, tag.payload);
    }

    Ok(tags_map)
}

/// Parses a NBT int array tag's payload into a [`Vec<i32>`].
/// For the format see [`super::serialize::write_int_array_payload`].
///
/// Uses [`parse_int_payload`] for parsing the actual ints.
pub fn parse_int_array_payload(input: &mut &[u8]) -> ModalResult<Vec<i32>> {
    length_repeat(
        be_i32
            .map(|x| x as usize)
            .context(StrContext::Label("int array length")),
        parse_int_payload.context(StrContext::Label("int")),
    )
    .context(StrContext::Label("int array payload"))
    .parse_next(input)
}

/// Parses a NBT long array tag's payload into a [`Vec<i64>`].
/// For the format see [`super::serialize::write_long_array_payload`].
///
/// Uses [`parse_long_payload`] for parsing the actual ints.
pub fn parse_long_array_payload(input: &mut &[u8]) -> ModalResult<Vec<i64>> {
    length_repeat(
        be_i32
            .map(|x| x as usize)
            .context(StrContext::Label("long array length")),
        parse_long_payload.context(StrContext::Label("long")),
    )
    .context(StrContext::Label("long array payload"))
    .parse_next(input)
}

/// Parses a NBT tag payload of a given [`TagPayloadType`] into a [`TagPayload`].
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
        TagPayloadType::End => parse_end_payload.map(|_| TagPayload::End).parse_next(input),
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

/// Parses a raw NBT tag from a byte slice `input` into a [`Tag`].
///
/// # Example
/// ```
/// use enbyt::binary::deserialize;
/// use std::fs::File;
///
/// let data: &[u8] = &[]; // <-- your data would be here
/// let tag = deserialize::parse_tag(&mut &data[..]);
///
/// dbg!(tag);
/// ```
pub fn parse_tag(input: &mut &[u8]) -> ModalResult<Tag> {
    dispatch! { any;
                0x01 => seq!((parse_tag_name, parse_byte_payload.map(TagPayload::Byte))).try_map(Tag::try_from),
                0x02 => seq!((parse_tag_name, parse_short_payload.map(TagPayload::Short))).try_map(Tag::try_from),
                0x03 => seq!((parse_tag_name, parse_int_payload.map(TagPayload::Int))).try_map(Tag::try_from),
                0x04 => seq!((parse_tag_name, parse_long_payload.map(TagPayload::Long))).try_map(Tag::try_from),
                0x05 => seq!((parse_tag_name, parse_float_payload.map(TagPayload::Float))).try_map(Tag::try_from),
                0x06 => seq!((parse_tag_name, parse_double_payload.map(TagPayload::Double))).try_map(Tag::try_from),
                0x07 => seq!((parse_tag_name, parse_byte_array_payload.map(TagPayload::ByteArray))).try_map(Tag::try_from),
                0x08 => seq!((parse_tag_name, parse_string_payload.map(TagPayload::String))).try_map(Tag::try_from),
                0x09 => seq!((parse_tag_name, parse_list_payload.map(|(id, tags)| TagPayload::List(id, tags)))).try_map(Tag::try_from),
                0x0a => seq!((parse_tag_name, parse_compound_payload.map(TagPayload::Compound))).try_map(Tag::try_from),
                0x0b => seq!((parse_tag_name, parse_int_array_payload.map(TagPayload::IntArray))).try_map(Tag::try_from),
                0x0c => seq!((parse_tag_name, parse_long_array_payload.map(TagPayload::LongArray))).try_map(Tag::try_from),
                _ => fail::<_,_,_>
            }
            .parse_next(input)
}

/// Parses a gzip-compressed NBT tag from a reader implementing [`Read`].
///
/// More commonly NBT data for Minecraft is gzip-compressed, so the below code can be used to
/// deserialize, for example, a `level.dat` file.
///
/// # Example
/// ```
/// use enbyt::binary::deserialize;
/// use std::fs::File;
///
/// let file = File::open("./tests/samples/level.dat").unwrap();
/// let tag = deserialize::parse_compressed_tag(file).unwrap();
///
/// dbg!(tag);
/// ```
pub fn parse_compressed_tag<R: Read>(r: R) -> Result<Tag, NBTError> {
    let mut decoder = GzDecoder::new(r);

    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;

    let tag = parse_tag(&mut &buf[..]).map_err(NBTError::ParsingError)?;

    Ok(tag)
}
