use enbyt::binary::deserialize::*;
use enbyt::binary::serialize::*;

use hegel::TestCase;
use hegel::generators as gs;

use crate::shared::generators::generate_tag;

mod shared;

#[hegel::test]
fn test_parse_string(tc: TestCase) {
    let str = tc.draw(gs::text());

    let mut buf = vec![0; 2 + str.len()];
    write_string(&mut buf, str.clone()).unwrap();

    assert_eq!(parse_string(&mut &buf[..]), Ok(str));
}

#[hegel::test]
fn test_parse_byte_payload(tc: TestCase) {
    let byte = tc.draw(gs::integers::<i8>());

    let mut buf = vec![0; 1];

    write_byte_payload(&mut buf, byte).unwrap();

    let parsed = parse_byte_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(byte));
}

#[hegel::test]
fn test_parse_short_payload(tc: TestCase) {
    let num = tc.draw(gs::integers::<i16>());

    let mut buf = vec![0; 2];

    write_short_payload(&mut buf, num).unwrap();

    let parsed = parse_short_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(num));
}

#[hegel::test]
fn test_parse_int_payload(tc: TestCase) {
    let num = tc.draw(gs::integers::<i32>());

    let mut buf = vec![0; 4];

    write_int_payload(&mut buf, num).unwrap();

    let parsed = parse_int_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(num));
}

#[hegel::test]
fn test_parse_long_payload(tc: TestCase) {
    let num = tc.draw(gs::integers::<i64>());

    let mut buf = vec![0; 8];

    write_long_payload(&mut buf, num).unwrap();

    let parsed = parse_long_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(num));
}

#[hegel::test]
fn test_parse_float_payload(tc: TestCase) {
    let num = tc.draw(gs::floats::<f32>());

    let mut buf = vec![0; 4];

    write_float_payload(&mut buf, num).unwrap();

    let parsed = parse_float_payload(&mut &buf[..]);

    if let Ok(parsed_val) = parsed {
        assert!(num.total_cmp(&parsed_val).is_eq());
    }
}

#[hegel::test]
fn test_parse_double_payload(tc: TestCase) {
    let num = tc.draw(gs::floats::<f64>());

    let mut buf = vec![0; 8];

    write_double_payload(&mut buf, num).unwrap();

    let parsed = parse_double_payload(&mut &buf[..]);

    if let Ok(parsed_val) = parsed {
        assert!(num.total_cmp(&parsed_val).is_eq());
    }
}

#[hegel::test]
fn test_parse_byte_array_payload(tc: TestCase) {
    let arr = tc.draw(gs::vecs(gs::integers::<u8>()));

    let mut buf = vec![0; 4 + arr.len()];

    write_byte_array_payload(&mut buf, &arr).unwrap();

    let parsed = parse_byte_array_payload(&mut &buf[..]);

    assert_eq!(parsed, Ok(arr));
}

#[hegel::test]
fn test_parse_tag(tc: TestCase) {
    let tag = tc.draw(generate_tag());

    // generously sized buffer because we really don't know how big the data from hegel is gonna be
    let mut buf = vec![0; 256];

    write_tag(&mut buf, tag.clone()).unwrap();

    let parsed = parse_tag(&mut &buf[..]);

    assert_eq!(parsed, Ok(tag));
}
