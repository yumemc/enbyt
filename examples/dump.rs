use std::{env, fs::File};

use enbyt::binary::deserialize::parse_compressed_tag;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let path = args
        .get(1)
        .expect("pass a file path as an argument (try tests/samples/level.dat)");

    let file = File::open(path).expect("couldn't read file");
    let tag = parse_compressed_tag(file).expect("couldn't parse tag");

    dbg!(tag);
}
