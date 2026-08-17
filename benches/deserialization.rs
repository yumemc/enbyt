use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use enbyt::binary::deserialize;

fn criterion_benchmark(c: &mut Criterion) {
    let bytes_compressed = include_bytes!("../tests/samples/level.dat");
    let bytes_raw = include_bytes!("../tests/samples/level.dat.raw");

    c.bench_function("deserialize compressed", |b| {
        b.iter(|| {
            black_box(deserialize::parse_compressed_tag(&bytes_compressed[..])).unwrap();
        })
    });

    c.bench_function("deserialize raw", |b| {
        b.iter(|| {
            black_box(deserialize::parse_tag(&mut &bytes_raw[..])).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
