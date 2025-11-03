use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use metadata_audio::Metadata;
use std::path::Path;

/*fn bench_parse_flac_bytes(c: &mut Criterion) {
    let flac_path = Path::new("test_data/test.flac");
    let data = std::fs::read(flac_path).unwrap();

    c.bench_function("parse_flac_from_bytes", |b| {
        b.iter(|| {
            let _ = Metadata::from_bytes(&data, SupportedFormats::Flac).unwrap();
        });
    });
}*/

fn bench_read_flac_from_disk(c: &mut Criterion) {
    let source = Path::new("test_data/test.flac");
    let size = std::fs::metadata(source).unwrap().len();

    let mut group = c.benchmark_group("flac_read_disk");
    group.throughput(Throughput::Bytes(size));

    group.bench_function("no_copy", |b| {
        b.iter(|| {
            let _ = Metadata::from_file(&source).unwrap();
        });
    });

    group.finish();
}

fn bench_read_many_flacs_from_disk(c: &mut Criterion) {
    let ssd_files = recursive_dir(&Path::new("/home/akira/Music/music/"));
    println!("{} files to be parsed from SSD...", ssd_files.len());

    let hdd_files = recursive_dir(&Path::new("/run/media/akira/3TB/StreamripDownloads/"));
    println!("{} files to be parsed from HDD...", hdd_files.len());

    let mut group = c.benchmark_group("read_many_flacs");
    group.sample_size(10);

    group.throughput(Throughput::Elements(ssd_files.len() as u64));
    group.bench_function("ssd", |b| {
        b.iter(|| {
            let _ = Metadata::from_files(&ssd_files).unwrap();
        });
    });

    /*group.throughput(Throughput::Elements(hdd_files.len() as u64));
    group.bench_function("hdd", |b| {
        b.iter(|| {
            let _ = Metadata::from_files(&hdd_files).unwrap();
        });
    });*/

    group.finish();
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
}

criterion_group!(
    benches,
    bench_read_flac_from_disk,
    bench_read_many_flacs_from_disk
);
criterion_main!(benches);
