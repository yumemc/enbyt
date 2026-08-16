use enbyt::binary::serialize::write_string;
use hegel::TestCase;
use hegel::generators as gs;

#[hegel::test]
fn test_write_string_length(tc: TestCase) {
    let str = tc.draw(gs::text());

    let mut buf = Vec::new();
    write_string(&mut buf, str.clone()).unwrap();

    let len_bytes = &buf[..2];
    let len = u16::from_be_bytes(len_bytes.try_into().unwrap()) as usize;

    assert_eq!(len, str.len());
}

#[hegel::test]
fn test_write_string_value(tc: TestCase) {
    let str = tc.draw(gs::text());

    let mut buf = Vec::new();
    write_string(&mut buf, str.clone()).unwrap();

    let str_bytes = &buf[2..];
    let str_decoded = String::from_utf8(str_bytes.into());

    assert_eq!(str_decoded, Ok(str));
}
