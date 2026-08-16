use enbyt::{
    Tag, TagPayload,
    binary::{
        deserialize::{parse_string_payload, parse_tag},
        serialize::{write_byte_payload, write_string_payload, write_tag},
    },
};

use std::{assert_matches, collections::HashMap};

#[test]
fn test_empty_tag() {
    let mut input: &[u8] = &[0x00u8];

    assert_eq!(
        parse_tag(&mut input),
        Ok(Tag::new(None, TagPayload::Empty).unwrap())
    );
}

#[test]
fn test_heterogenous_list() {
    let mut buf = Vec::new();

    let list = Tag::new(
        Some("list".to_string()),
        TagPayload::List(
            0x03,
            vec![
                Tag::new(Some("a".to_string()), TagPayload::String("aa".to_string())).unwrap(),
                Tag::new(Some("b".to_string()), TagPayload::Byte(4)).unwrap(),
            ],
        ),
    )
    .unwrap();

    assert_matches!(
        write_tag(&mut buf, list),
        Err(enbyt::NBTError::UnexpectedType(0x03, _))
    );
}

#[test]
fn test_homogenous_list() {
    let mut buf = Vec::new();

    let list = Tag::new(
        Some("list".to_string()),
        TagPayload::List(
            0x03,
            vec![
                Tag::new(Some("a".to_string()), TagPayload::Int(3)).unwrap(),
                Tag::new(Some("b".to_string()), TagPayload::Int(4)).unwrap(),
            ],
        ),
    )
    .unwrap();

    assert!(write_tag(&mut buf, list).is_ok());
}

#[test]
fn test_byte_payload_length() {
    let mut buf = Vec::new();

    assert_matches!(write_byte_payload(&mut buf, 3), Ok(1));
}

#[test]
fn test_empty_string_tag() {
    let mut buf = Vec::new();

    let tag = Tag::new(Some("".to_string()), TagPayload::String("".to_string())).unwrap();
    write_tag(&mut buf, tag.clone()).unwrap();

    assert_eq!(buf, vec![0x08, 0x00, 0x00, 0x00, 0x00]);

    let parsed = parse_tag(&mut &buf[..]);

    assert_eq!(parsed, Ok(tag));
}

#[test]
fn test_empty_compound_tag() {
    let mut buf = Vec::new();

    let tag = Tag::new(Some("".to_string()), TagPayload::Compound(HashMap::new())).unwrap();
    write_tag(&mut buf, tag.clone()).unwrap();

    assert_eq!(buf, vec![0x0a, 0x00, 0x00, 0x00]);

    let parsed = parse_tag(&mut &buf[..]);

    assert_eq!(parsed, Ok(tag));
}
