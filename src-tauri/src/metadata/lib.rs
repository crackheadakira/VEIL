mod error;
mod flac;

use std::{collections::HashMap, path::PathBuf};

use futures::future::join_all;
use serde::Serialize;
use specta::Type;
use tokio;

#[derive(Debug, Serialize, Clone, Type)]
pub struct Metadata {
    pub duration: f32,
    pub album: String,
    pub artist: String,
    pub name: String,
    pub file_path: String,
    pub album_type: String,
    pub year: u16,
    pub track_number: u16,
    pub picture_data: Vec<u8>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            duration: 0.0,
            album: String::new(),
            artist: String::new(),
            name: String::new(),
            file_path: String::new(),
            album_type: String::new(),
            year: 0,
            track_number: 0,
            picture_data: Vec::new(),
        }
    }

    fn from_flac(file: flac::Flac) -> Metadata {
        let vc = file.vorbis_comment.unwrap();

        Metadata {
            duration: file.stream_info.duration,
            album: get_field_value(&vc.fields, "ALBUM"),
            artist: get_field_value(&vc.fields, "ARTIST"),
            name: get_field_value(&vc.fields, "TITLE"),
            file_path: file.file_path,
            album_type: get_field_value(&vc.fields, "ALBUMTYPE"),
            year: get_field_value(&vc.fields, "DATE").parse().unwrap_or(0),
            track_number: get_field_value(&vc.fields, "TRACKNUMBER")
                .parse()
                .unwrap_or(0),
            picture_data: file.picture.unwrap().data,
        }
    }

    pub fn from_file(path: &std::path::Path) -> error::Result<Metadata> {
        let ext = path.extension().unwrap().to_str().unwrap();

        match ext {
            "flac" => {
                let file = flac::Flac::new(path)?;
                Ok(Metadata::from_flac(file))
            }
            _ => Err("Unsupported file type".into()),
        }
    }

    /*pub fn from_files(
        file_paths: &Vec<std::path::PathBuf>,
        file_extension: &str,
    ) -> error::Result<Vec<Metadata>> {
        match file_extension {
            "flac" => {
                let files: Vec<flac::Flac> = file_paths
                    .par_iter()
                    .map(|path| flac::Flac::new(path).unwrap())
                    .collect();

                Ok(files.into_iter().map(Metadata::from_flac).collect())
            }
            _ => Err("Unsupported file type".into()),
        }
    }*/

    pub async fn from_files(
        file_paths: &Vec<PathBuf>,
        file_extension: &str,
    ) -> error::Result<Vec<Metadata>> {
        match file_extension {
            "flac" => {
                let tasks: Vec<_> = file_paths
                    .iter()
                    .map(|path| {
                        let path = path.clone(); // Clone the PathBuf so the async block owns it
                        tokio::spawn(async move {
                            // This will run the FLAC loading asynchronously
                            let file = flac::Flac::new(&path).unwrap(); // Unwrap or handle error as needed
                            Metadata::from_flac(file) // Convert FLAC to Metadata
                        })
                    })
                    .collect();

                // Await all tasks concurrently
                let results: Vec<_> = join_all(tasks).await;

                // Collect results and handle errors
                let metadata: Vec<Metadata> = results.into_iter().collect::<Result<_, _>>()?;
                Ok(metadata)
            }
            _ => Err("Unsupported file type".into()),
        }
    }
}

fn get_field_value(fields: &HashMap<String, Vec<String>>, key: &str) -> String {
    fields
        .get(key)
        .and_then(|v| v.first())
        .map(|s| s.to_string()) // Converts &str to String
        .unwrap_or_else(|| "Unknown".to_string()) // Default value if not found
}
