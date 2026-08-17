use enbyt::binary::deserialize::parse_compressed_tag;

#[test]
fn test_parse_level_sample() {
    let bytes = include_bytes!("samples/level.dat");
    let tag = parse_compressed_tag(&bytes[..]);

    assert!(tag.is_ok());
}
