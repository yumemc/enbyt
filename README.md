# `enbyt`

[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/enbyt.svg)
[![Crates Status](https://img.shields.io/crates/v/enbyt.svg)][Crates.io]

A library for Serializing/Deserializing NBT data. NBT (Named Binary Tag) is a binary data format
used by Minecraft to encode various kinds of information in worlds.

NBT Format Reference: <https://minecraft.wiki/w/NBT_format>

# Examples

## Loading a `level.dat`
```rust
use enbyt::binary::deserialize;
use std::fs::File;

let file = File::open("./tests/samples/level.dat").unwrap();
let tag = deserialize::parse_compressed_tag(file).unwrap();

dbg!(tag);
```

## Writing an NBT File
```rust
use enbyt::{Tag, TagPayload, binary::serialize};

let tag = Tag::new("foo".to_string(), TagPayload::String("bar".to_string())).unwrap();
let mut buf = Vec::new();

serialize::write_compressed_tag(buf, &tag).unwrap();
```

[Crates.io]: https://crates.io/crates/enbyt
[Documentation]: https://docs.rs/enbyt
