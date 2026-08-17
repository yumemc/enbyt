use enbyt::binary::deserialize::parse_compressed_tag;

fn main() {
    let bytes = include_bytes!("level.dat");
    let tag = parse_compressed_tag(&bytes[..]).expect("couldn't parse tag");

    dbg!(tag);
}
