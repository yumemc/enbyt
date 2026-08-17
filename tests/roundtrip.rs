use enbyt::binary::deserialize::*;
use enbyt::binary::serialize::*;

use hegel::TestCase;
use hegel::generators as gs;

use crate::shared::generators::generate_tag;

mod shared;

#[hegel::test]
fn test_string_roundtrip(tc: TestCase) {
    let str = tc.draw(gs::text());

    let mut buf = Vec::new();
    write_string(&mut buf, &str).unwrap();

    assert_eq!(parse_string(&mut &buf[..]), Ok(str));
}

#[hegel::test]
fn test_byte_payload_roundtrip(tc: TestCase) {
    let byte = tc.draw(gs::integers::<i8>());

    let mut buf = Vec::new();

    write_byte_payload(&mut buf, byte).unwrap();

    let parsed = parse_byte_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(byte));
}

#[hegel::test]
fn test_short_payload_roundtrip(tc: TestCase) {
    let num = tc.draw(gs::integers::<i16>());

    let mut buf = Vec::new();

    write_short_payload(&mut buf, num).unwrap();

    let parsed = parse_short_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(num));
}

#[hegel::test]
fn test_int_payload_roundtrip(tc: TestCase) {
    let num = tc.draw(gs::integers::<i32>());

    let mut buf = Vec::new();

    write_int_payload(&mut buf, num).unwrap();

    let parsed = parse_int_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(num));
}

#[hegel::test]
fn test_long_payload_roundtrip(tc: TestCase) {
    let num = tc.draw(gs::integers::<i64>());

    let mut buf = Vec::new();

    write_long_payload(&mut buf, num).unwrap();

    let parsed = parse_long_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(num));
}

#[hegel::test]
fn test_float_payload_roundtrip(tc: TestCase) {
    let num = tc.draw(gs::floats::<f32>());

    let mut buf = Vec::new();

    write_float_payload(&mut buf, num).unwrap();

    let parsed = parse_float_payload(&mut &buf[..]);

    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap().to_bits(), num.to_bits());
}

#[hegel::test]
fn test_double_payload_roundtrip(tc: TestCase) {
    let num = tc.draw(gs::floats::<f64>());

    let mut buf = Vec::new();

    write_double_payload(&mut buf, num).unwrap();

    let parsed = parse_double_payload(&mut &buf[..]);

    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap().to_bits(), num.to_bits());
}

#[hegel::test]
fn test_byte_array_payload_roundtrip(tc: TestCase) {
    let arr = tc.draw(gs::vecs(gs::integers::<i8>()));

    let mut buf = Vec::new();

    write_byte_array_payload(&mut buf, &arr).unwrap();

    let parsed = parse_byte_array_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(arr));
}

#[hegel::test]
fn test_tag_roundtrip(tc: TestCase) {
    let tag = tc.draw(generate_tag());

    let mut buf = Vec::new();

    // we only do the assertion if we wrote successfully, because, well, if we didn't write
    // succesfully it means the generate data broke a constraint (which our data generator allows)
    if write_tag(&mut buf, &tag).is_ok() {
        let parsed = parse_tag(&mut &buf[..]);

        assert_eq!(parsed, Ok(tag));
    }
}

#[hegel::test]
fn test_compressed_tag_roundtrip(tc: TestCase) {
    let tag = tc.draw(generate_tag());

    let mut buf = Vec::new();

    if write_compressed_tag(&mut buf, &tag).is_ok() {
        let parsed = parse_compressed_tag(&mut &buf[..]);

        assert_eq!(parsed, Ok(tag));
    }
}
