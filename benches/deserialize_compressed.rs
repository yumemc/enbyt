use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use enbyt::binary::deserialize;

fn criterion_benchmark(c: &mut Criterion) {
    let bytes = include_bytes!("../tests/samples/level.dat");

    c.bench_function("deserialize compressed", |b| {
        b.iter(|| {
            let _ = black_box(deserialize::parse_compressed_tag(&bytes[..]));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
