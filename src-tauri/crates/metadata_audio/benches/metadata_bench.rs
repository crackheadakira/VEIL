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

/*fn bench_read_many_flacs(c: &mut Criterion) {j
    let ssd_files = recursive_dir(&Path::new("/home/akira/Music/music/"));

    println!("{} files to be parsed from SSD...", ssd_files.len());

    let hdd_files = recursive_dir(&Path::new("/run/media/akira/3TB/StreamripDownloads/"));

    println!("{} files to be parsed from HDD...", hdd_files.len());

    let mut group = c.benchmark_group("read_many_flacs");

    group.throughput(Throughput::Elements(ssd_files.len() as u64));
    group.bench_function("SSD normal", |b| {
        b.iter(|| {
            let _ = Metadata::from_files(&ssd_files, false).unwrap();
        })
    });

    group.sample_size(10);

    group.throughput(Throughput::Elements(hdd_files.len() as u64));
    group.bench_function("HDD normal", |b| {
        b.iter(|| {
            let _ = Metadata::from_files(&hdd_files, false).unwrap();
        })
    });
}

fn bench_read_many_flacs_smart(c: &mut Criterion) {
    let ssd_files = Metadata::collect_album_files_for_smart(&Path::new("/home/akira/Music/music/"))
        .expect("Error getting SSD files");

    let ssd_total_files = ssd_files.iter().map(|album| album.len()).sum();
    println!("{} files to be parsed from SSD...", ssd_total_files);

    let hdd_files = Metadata::collect_album_files_for_smart(&Path::new(
        "/run/media/akira/3TB/StreamripDownloads/",
    ))
    .expect("Error getting HDD files");

    let hdd_total_files = hdd_files.iter().map(|album| album.len()).sum();
    println!("{} files to be parsed from HDD...", hdd_total_files);

    let mut group = c.benchmark_group("read_many_flacs");

    group.throughput(Throughput::Elements(ssd_total_files as u64));
    group.bench_function("SSD smart", |b| {
        b.iter(|| {
            let _ =
                Metadata::from_files_smart::<fn(usize)>(&ssd_files, ssd_total_files, None).unwrap();
        })
    });

    group.sample_size(10);

    group.throughput(Throughput::Elements(hdd_total_files as u64));
    group.bench_function("HDD smart", |b| {
        b.iter(|| {
            let _ =
                Metadata::from_files_smart::<fn(usize)>(&hdd_files, hdd_total_files, None).unwrap();
        })
    });
}

fn recursive_dir(path: &Path) -> Vec<std::path::PathBuf> {
    let paths = std::fs::read_dir(path).unwrap();
    let mut tracks = Vec::new();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            tracks.extend(recursive_dir(&path));
        } else {
            let extension = path.extension().unwrap();
            if extension != "mp3" && extension != "flac" {
                continue;
            }

            tracks.push(path);
        }
    }

    tracks
}*/

criterion_group!(
    benches,
    bench_read_flac_from_bytes,
    bench_read_flac_from_bytes_mass,
    // bench_read_many_flacs,
    // bench_read_many_flacs_smart
);
criterion_main!(benches);
