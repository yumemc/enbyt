use std::fs::File;

use enbyt::binary::deserialize::parse_compressed_tag;

fn main() {
    let file = include_bytes!("level.dat");
    let tag = parse_compressed_tag(&file[..]).expect("couldn't parse tag");

    dbg!(tag);
}
