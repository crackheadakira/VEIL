use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use metadata_audio::Metadata;
use std::fs::{self};

fn bench_read_flac_from_bytes(c: &mut Criterion) {
    let data = fs::read("test_data/test.flac").unwrap();

    let mut group = c.benchmark_group("flac_parse_bytes");
    let mut buffer = Vec::with_capacity(1024 * 32);

    group.throughput(Throughput::Elements(1));
    group.bench_function("no_skip", |b| {
        b.iter(|| {
            buffer.clear();
            let _ = Metadata::from_bytes(
                &mut buffer,
                &data,
                metadata_audio::SupportedFormats::Flac,
                false,
            )
            .unwrap();
        });
    });

    group.throughput(Throughput::Elements(1));
    group.bench_function("skip", |b| {
        b.iter(|| {
            buffer.clear();
            let _ = Metadata::from_bytes(
                &mut buffer,
                &data,
                metadata_audio::SupportedFormats::Flac,
                true,
            )
            .unwrap();
        });
    });

    group.finish();
}

fn bench_read_id3_from_bytes(c: &mut Criterion) {
    let data = fs::read("test_data/test.mp3").unwrap();

    let mut group = c.benchmark_group("id3_parse_bytes");
    let mut buffer = Vec::with_capacity(1024 * 32);

    group.throughput(Throughput::Elements(1));
    group.bench_function("no_skip", |b| {
        b.iter(|| {
            buffer.clear();
            let _ = Metadata::from_bytes(
                &mut buffer,
                &data,
                metadata_audio::SupportedFormats::ID3,
                false,
            )
            .unwrap();
        });
    });

    group.finish();
}

fn bench_read_flac_from_bytes_mass(c: &mut Criterion) {
    let data = fs::read("test_data/test.flac").unwrap();

    let mut group = c.benchmark_group("flac_parse_bytes_mass");
    let mut buffer = Vec::with_capacity(1024 * 32);

    group.throughput(Throughput::Elements(100));
    group.bench_function("no_skip, clear (100)", |b| {
        b.iter(|| {
            for _ in 0..100 {
                buffer.clear();
                let _ = Metadata::from_bytes(
                    &mut buffer,
                    &data,
                    metadata_audio::SupportedFormats::Flac,
                    false,
                )
                .unwrap();
            }
        });
    });

    group.throughput(Throughput::Elements(100));
    group.bench_function("skip, clear (100)", |b| {
        b.iter(|| {
            for _ in 0..100 {
                buffer.clear();
                let _ = Metadata::from_bytes(
                    &mut buffer,
                    &data,
                    metadata_audio::SupportedFormats::Flac,
                    true,
                )
                .unwrap();
            }
        });
    });

    group.throughput(Throughput::Elements(100));
    group.bench_function("no_skip, new (100)", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut buffer = Vec::with_capacity(1024 * 32);
                let _ = Metadata::from_bytes(
                    &mut buffer,
                    &data,
                    metadata_audio::SupportedFormats::Flac,
                    false,
                )
                .unwrap();
            }
        });
    });

    group.throughput(Throughput::Elements(100));
    group.bench_function("skip, new (100)", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut buffer = Vec::with_capacity(1024 * 32);
                let _ = Metadata::from_bytes(
                    &mut buffer,
                    &data,
                    metadata_audio::SupportedFormats::Flac,
                    true,
                )
                .unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_read_flac_from_bytes,
    bench_read_id3_from_bytes,
    // bench_read_flac_from_bytes_mass,
);
criterion_main!(benches);
