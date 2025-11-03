use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use metadata_audio::Metadata;
use std::path::Path;

fn bench_read_flac_from_disk(c: &mut Criterion) {
    let source = Path::new("test_data/test.flac");
    let size = std::fs::metadata(source).unwrap().len();

    let mut group = c.benchmark_group("flac_read_disk");
    group.throughput(Throughput::Bytes(size));

    group.bench_function("no_skip", |b| {
        b.iter(|| {
            let _ = Metadata::from_file(&source, false).unwrap();
        });
    });

    group.bench_function("skip", |b| {
        b.iter(|| {
            let _ = Metadata::from_file(&source, true).unwrap();
        });
    });

    group.finish();
}

fn bench_read_many_flacs_smart(c: &mut Criterion) {
    let ssd_files = collect_album_tracks(&Path::new("/home/akira/Music/music/"));
    let ssd_total_files = ssd_files.iter().map(|album| album.len()).sum();
    println!("{} files to be parsed from SSD...", ssd_total_files);

    let hdd_files = collect_album_tracks(&Path::new("/run/media/akira/3TB/StreamripDownloads/"));
    let hdd_total_files = hdd_files.iter().map(|album| album.len()).sum();
    println!("{} files to be parsed from HDD...", hdd_total_files);

    let mut group = c.benchmark_group("read_many_flacs");
    group.sample_size(10);

    group.bench_function("SSD smart", |b| {
        b.iter(|| {
            do_test_thing(&ssd_files, ssd_total_files);
        })
    });

    group.bench_function("HDD smart", |b| {
        b.iter(|| {
            do_test_thing(&hdd_files, hdd_total_files);
        })
    });
}

fn do_test_thing(files: &Vec<Vec<std::path::PathBuf>>, total_files: usize) {
    let mut all_metadata = Vec::with_capacity(total_files);

    for album_tracks in files.iter() {
        let first_track = &album_tracks[0];
        let first_metadata = Metadata::from_file(first_track, false).unwrap();

        for track in album_tracks[1..].iter() {
            let metadata = Metadata::from_file(track, true);

            match metadata {
                Ok(mut metadata) => {
                    if metadata.picture_data.is_none() {
                        metadata.picture_data = first_metadata.picture_data.clone();
                    }

                    all_metadata.push(metadata);
                }

                Err(_) => continue,
            }
        }

        all_metadata.push(first_metadata);
    }
}

fn collect_album_tracks(path: &Path) -> Vec<Vec<std::path::PathBuf>> {
    let mut albums = Vec::new();

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            albums.extend(collect_album_tracks(&path));
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("mp3") || ext.eq_ignore_ascii_case("flac") {
                // Find album by parent
                if let Some(album_idx) = albums.iter().position(|a| a[0].parent() == path.parent())
                {
                    albums[album_idx].push(path);
                } else {
                    albums.push(vec![path]);
                }
            }
        }
    }

    albums
}

criterion_group!(
    benches,
    bench_read_flac_from_disk,
    bench_read_many_flacs_smart
);
criterion_main!(benches);
