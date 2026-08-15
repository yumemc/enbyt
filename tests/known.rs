use enbyt::{
    Tag, TagPayload,
    binary::{deserialize::parse_tag, serialize::write_tag},
};

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
    let mut buf = vec![0; 12];

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

    assert_eq!(
        write_tag(&mut buf, list),
        Err(enbyt::NBTError::UnexpectedType(
            0x03,
            Tag::new(Some("a".to_string()), TagPayload::String("aa".to_string())).unwrap(),
        ))
    );
}

#[test]
fn test_homogenous_list() {
    let mut buf = vec![0; 12];

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
