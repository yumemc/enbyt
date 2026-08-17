use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use enbyt::binary::deserialize;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("deserialize compressed", |b| {
        b.iter(|| {
            let bytes = include_bytes!("../tests/samples/level.dat");

            black_box(deserialize::parse_compressed_tag(&bytes[..])).unwrap();
        })
    });

    c.bench_function("deserialize raw", |b| {
        b.iter(|| {
            let bytes = include_bytes!("../tests/samples/level.dat.raw");

            black_box(deserialize::parse_tag(&mut &bytes[..])).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
