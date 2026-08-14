use enbyt::{Tag, TagPayload, binary::deserialize::parse_tag};

#[test]
fn test_empty_tag() {
    let mut input: &[u8] = &[0x00u8];

    assert_eq!(
        parse_tag(&mut input),
        Ok(Tag::new(None, TagPayload::Empty).unwrap())
    );
}
