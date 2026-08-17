//! Contains the code for serializing binary NBT data.
use std::io::Write;

use flate2::{Compression, write::GzEncoder};

use crate::{NBTError, Tag, TagPayload, TagPayloadType};

/// Writes a string slice `str` into `w`.
///
/// Returns the amount of bytes written.
///
/// The format is:
/// - 2 byte integer (Big Endian) indicating the string's length
/// - the string's bytes encoded using UTF-8.
pub fn write_string<W: Write>(w: &mut W, str: &str) -> Result<usize, NBTError> {
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
pub fn write_tag_name<W: Write>(w: &mut W, name: &str) -> Result<usize, NBTError> {
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
pub fn write_byte_array_payload<W: Write>(w: &mut W, arr: Vec<i8>) -> Result<usize, NBTError> {
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
pub fn write_string_payload<W: Write>(w: &mut W, str: &str) -> Result<usize, NBTError> {
    write_string(w, str)
}

/// Writes a NBT list payload `list` into `w`.
///
/// Returns the amount of bytes written.
///
/// The format is:
/// - 1 byte for the ID of the type of the list's contents
/// - 4 bytes for the length
/// - every payload in the list
pub fn write_list_payload<W: Write>(
    w: &mut W,
    (tag_type, list): (TagPayloadType, Vec<TagPayload>),
) -> Result<usize, NBTError> {
    let mut written = 0;

    let type_id = tag_type as u8;

    // ensure all items are of the same type
    // let first_different = list.iter().find(|tag| tag.type_id() != type_id);
    //
    // if let Some(tag) = first_different {
    //     return Err(NBTError::UnexpectedType(type_id, tag.clone()));
    // }

    let length = list.len() as i32;

    written += w.write(&[type_id])?;
    written += w.write(&length.to_be_bytes())?;

    for item in list {
        written += write_payload(w, item)?;
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
pub fn write_compound_payload<W: Write>(w: &mut W, value: Vec<Tag>) -> Result<usize, NBTError> {
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
pub fn write_int_array_payload<W: Write>(w: &mut W, arr: Vec<i32>) -> Result<usize, NBTError> {
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
pub fn write_long_array_payload<W: Write>(w: &mut W, arr: Vec<i64>) -> Result<usize, NBTError> {
    let mut written = 0;

    let length = arr.len() as i32;

    written += w.write(&length.to_be_bytes())?;

    for item in arr {
        written += write_long_payload(w, item)?;
    }

    Ok(written)
}

pub fn write_payload<W: Write>(w: &mut W, payload: TagPayload) -> Result<usize, NBTError> {
    match payload {
        TagPayload::Byte(value) => write_byte_payload(w, value),
        TagPayload::Short(value) => write_short_payload(w, value),
        TagPayload::Int(value) => write_int_payload(w, value),
        TagPayload::Long(value) => write_long_payload(w, value),
        TagPayload::Float(value) => write_float_payload(w, value),
        TagPayload::Double(value) => write_double_payload(w, value),
        TagPayload::String(value) => write_string_payload(w, &value),
        TagPayload::List(tag_type, value) => write_list_payload(w, (tag_type, value)),
        TagPayload::Compound(value) => write_compound_payload(w, value.into_values().collect()),
        TagPayload::ByteArray(value) => write_byte_array_payload(w, value),
        TagPayload::IntArray(items) => write_int_array_payload(w, items),
        TagPayload::LongArray(items) => write_long_array_payload(w, items),
    }
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
///
/// # Example
/// ```
/// use enbyt::{Tag, TagPayload, binary::serialize};
///
/// let tag = Tag::new("foo".to_string(), TagPayload::String("bar".to_string())).unwrap();
/// let mut buf = Vec::new();
///
/// serialize::write_tag(&mut buf, tag).unwrap();
/// ```
pub fn write_tag<W: Write>(w: &mut W, tag: Tag) -> Result<usize, NBTError> {
    let mut written = 0;

    let tag_type_id = tag.type_id();

    written += w.write(&tag_type_id.to_be_bytes())?;

    written += write_string(w, &tag.name)?;

    written += write_payload(w, tag.payload)?;

    Ok(written)
}

/// Writes a gzip-compressed NBT tag into a writer implementing [`Write`].
///
/// # Example
/// ```
/// use enbyt::{Tag, TagPayload, binary::serialize};
///
/// let tag = Tag::new("foo".to_string(), TagPayload::String("bar".to_string())).unwrap();
/// let mut buf = Vec::new();
///
/// serialize::write_compressed_tag(buf, tag).unwrap();
/// ```
pub fn write_compressed_tag<W: Write>(w: W, tag: Tag) -> Result<(), NBTError> {
    let mut encoder = GzEncoder::new(w, Compression::default());

    write_tag(&mut encoder, tag)?;

    // I believe we *don't* need to also call encoder.flush(), but I may be wrong.
    encoder.try_finish()?;

    // TODO: it's a shame not to return the amount of bytes written, how do we do that?
    Ok(())
}
