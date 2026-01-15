use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use mpsquish::{compacted_stream_to_json, pack_msgpack_stream};

fn json2msgpack(json: &str) -> Vec<u8> {
    let json = serde_json::from_str::<serde_json::Value>(json).unwrap();
    rmp_serde::to_vec_named(&json).unwrap()
}

fn pack_msgpack_stream_bench(c: &mut Criterion) {
    let data = json2msgpack(include_str!("./citm_catalog.json"));
    c.bench_function("pack citm catalog", |b| {
        b.iter(|| {
            let mut interner = lasso::Rodeo::new();
            let mut out = Vec::with_capacity(data.len());
            pack_msgpack_stream(&data[..], &mut interner, &mut out);
        });
    });

    let mut interner = lasso::Rodeo::new();
    let mut out = Vec::with_capacity(data.len());
    pack_msgpack_stream(&data[..], &mut interner, &mut out);
    println!("unpacked size: {} bytes", data.len());
    println!("packed size: {} bytes", out.len());
}

fn unpack_msgpack_stream_bench(c: &mut Criterion) {
    let data = json2msgpack(include_str!("./citm_catalog.json"));

    let mut interner = lasso::Rodeo::new();
    let mut packed_data = Vec::with_capacity(data.len());
    pack_msgpack_stream(&data[..], &mut interner, &mut packed_data);
    let interner = interner.into_resolver();

    let mut group = c.benchmark_group("unpack msgpack -> json");
    group.bench_function("serde_transcode (rmp/serde_json)", |b| {
        b.iter(|| {
            let mut reader = rmp_serde::Deserializer::from_read_ref(&data[..]);
            let out = Vec::with_capacity(data.len() * 2);
            let mut writer = serde_json::Serializer::new(out);
            serde_transcode::transcode(&mut reader, &mut writer).unwrap();
        });
    });
    group.bench_function("mpsquish (interned rmp/nyoom_json)", |b| {
        b.iter(|| {
            let mut out = String::with_capacity(data.len() * 2);
            compacted_stream_to_json(&packed_data[..], &interner, &mut out);
        });
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = pack_msgpack_stream_bench,
    unpack_msgpack_stream_bench
);
criterion_main!(benches);
