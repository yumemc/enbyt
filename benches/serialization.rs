use criterion::{Criterion, criterion_group, criterion_main};
use enbyt::binary::{deserialize::parse_tag, serialize};

fn criterion_benchmark(c: &mut Criterion) {
    let bytes = include_bytes!("../tests/samples/level.dat.raw");
    let tag = parse_tag(&mut &bytes[..]).unwrap();

    c.bench_function("serialize compressed", |b| {
        b.iter(|| {
            let mut buf: Vec<u8> = Vec::new();
            serialize::write_compressed_tag(&mut buf, &tag).unwrap();
        })
    });

    c.bench_function("serialize raw", |b| {
        b.iter(|| {
            let mut buf: Vec<u8> = Vec::new();
            serialize::write_tag(&mut buf, &tag).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
