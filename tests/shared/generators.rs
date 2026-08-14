use enbyt::Tag;
use enbyt::TagPayload;
use hegel::TestCase;
use hegel::generators as gs;

#[hegel::composite]
pub fn generate_tag_payload(tc: &TestCase) -> TagPayload {
    let id = tc.draw(gs::integers::<u8>().min_value(0).max_value(12));

    match id {
        0x00 => TagPayload::Empty,
        0x01 => TagPayload::Byte(tc.draw(gs::integers::<i8>())),
        0x02 => TagPayload::Short(tc.draw(gs::integers::<i16>())),
        0x03 => TagPayload::Int(tc.draw(gs::integers::<i32>())),
        0x04 => TagPayload::Long(tc.draw(gs::integers::<i64>())),
        0x05 => TagPayload::Float(tc.draw(gs::floats::<f32>())),
        0x06 => TagPayload::Double(tc.draw(gs::floats::<f64>())),
        0x07 => TagPayload::ByteArray(tc.draw(gs::vecs(gs::integers::<u8>()))),
        0x08 => TagPayload::String(tc.draw(gs::text())),
        0x09 => TagPayload::List(
            tc.draw(gs::integers::<i8>()),
            tc.draw(gs::vecs(generate_tag())),
        ),
        0x0a => TagPayload::Compound(tc.draw(gs::hashmaps(gs::text(), generate_tag()))),
        0x0b => TagPayload::IntArray(tc.draw(gs::vecs(gs::integers::<i32>()))),
        0x0c => TagPayload::LongArray(tc.draw(gs::vecs(gs::integers::<i64>()))),
        _ => unreachable!(),
    }
}

#[hegel::composite]
pub fn generate_tag(tc: &TestCase) -> Tag {
    let name = tc.draw(gs::optional(gs::text()));
    let payload = tc.draw(generate_tag_payload());

    Tag::new(name, payload)
}
