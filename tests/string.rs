use byteorder::BigEndian;
use byteorder::ByteOrder;
use enbyt::binary::serialize::write_string;
use hegel::TestCase;
use hegel::generators as gs;

#[hegel::test]
fn test_write_string_length(tc: TestCase) {
    let str = tc.draw(gs::text());

    let mut buf = vec![0; 2 + str.len()];
    write_string(&mut buf, str.clone());

    let len_bytes = &buf[..2];
    let len = BigEndian::read_u16(len_bytes) as usize;

    assert_eq!(len, str.len());
}

#[hegel::test]
fn test_write_string_value(tc: TestCase) {
    let str = tc.draw(gs::text());

    let mut buf = vec![0; 2 + str.len()];
    write_string(&mut buf, str.clone());

    let str_bytes = &buf[2..];
    let str_decoded = String::from_utf8(str_bytes.into());

    assert_eq!(str_decoded, Ok(str));
}
