mod flac;
mod id3;

use std::collections::HashMap;

#[derive(Debug, Clone)]
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
    pub features: Vec<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("Unsupported file type")]
    UnsupportedFileType,
    #[error("Invalid FLAC signature")]
    InvalidFlacSignature,
    #[error("Invalid ID3 signature")]
    InvalidId3Signature,
    #[error("Unsupported ID3 version")]
    UnsupportedId3Version,

    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Str(#[from] std::str::Utf8Error),
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
            features: Vec::new(),
        }
    }

    fn from_flac(file: flac::Flac) -> Metadata {
        let vc = file.vorbis_comment;

        let album_artist = get_field_value(&vc.fields, "ALBUMARTIST");
        let artists = get_field_value(&vc.fields, "ARTIST");

        let mut features = artists.replace(&album_artist, "");
        let mut features_vec = Vec::new();
        if features.len() > 0 {
            if features.starts_with(", ") {
                features.remove(0);
            }

            features_vec = features.split(", ").map(|s| s.trim().to_string()).collect();
        };

        Metadata {
            duration: file.stream_info.duration,
            album: get_field_value(&vc.fields, "ALBUM"),
            artist: get_field_value(&vc.fields, "ALBUMARTIST"),
            name: get_field_value(&vc.fields, "TITLE"),
            file_path: file.file_path,
            album_type: get_field_value(&vc.fields, "ALBUMTYPE"),
            year: get_field_value(&vc.fields, "YEAR").parse().unwrap_or(0),
            track_number: get_field_value(&vc.fields, "TRACKNUMBER")
                .parse()
                .unwrap_or(0),
            picture_data: file.picture.unwrap_or_default().data,
            features: features_vec,
        }
    }

    fn from_id3(file: id3::Id3) -> Metadata {
        Metadata {
            duration: 0.0,
            album: get_field_value(&file.text_frames, "TALB"),
            artist: get_field_value(&file.text_frames, "TPE1"),
            name: get_field_value(&file.text_frames, "TIT2"),
            file_path: file.file_path,
            album_type: "Unknown".to_string(),
            year: get_field_value(&file.text_frames, "TYER")
                .parse()
                .unwrap_or(0),
            track_number: get_field_value(&file.text_frames, "TRCK") //
                .split('/')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            picture_data: file.attached_picture.unwrap_or_default().picture_data,
            features: vec![],
        }
    }

    pub fn from_file(path: &std::path::Path) -> Result<Metadata, MetadataError> {
        let ext = path.extension().unwrap().to_str().unwrap();

        match ext {
            "flac" => {
                let file = flac::Flac::new(path)?;
                Ok(Metadata::from_flac(file))
            }
            "mp3" => {
                let file = id3::Id3::new(path)?;
                Ok(Metadata::from_id3(file))
            }
            _ => Err(MetadataError::UnsupportedFileType),
        }
    }

    pub fn from_files(file_paths: &[std::path::PathBuf]) -> Result<Vec<Metadata>, MetadataError> {
        let mut all_metadata = Vec::new();
        for path in file_paths {
            all_metadata.push(Metadata::from_file(path)?);
        }
        Ok(all_metadata)
    }
}

fn get_field_value(fields: &HashMap<String, String>, key: &str) -> String {
    fields
        .get(key)
        .unwrap_or(&String::from("Unknown"))
        .to_string()
}

pub fn read_n_bits<T>(bytes: &[u8], start_bit: usize, n_bits: usize) -> T
where
    T: Default + Copy + std::ops::Shl<u32, Output = T> + std::ops::BitOr<Output = T> + From<u8>,
{
    assert!(
        n_bits <= std::mem::size_of::<T>() * 8,
        "Cannot read more bits than fit in T."
    );
    let total_bits = bytes.len() * 8;
    assert!(start_bit + n_bits <= total_bits, "Not enough bits to read.");

    let mut value = T::default();
    for bit_index in 0..n_bits {
        let bit_position = start_bit + bit_index;
        let byte_index = bit_position / 8;
        let bit_in_byte = 7 - (bit_position % 8); // Most significant bit first
        let bit = (bytes[byte_index] >> bit_in_byte) & 1;
        value = (value << 1) | T::from(bit);
    }

    value
}

pub enum Endian {
    Big,
    Little,
}

pub fn u32_from_bytes(endian: Endian, bytes: &[u8], offset: &mut usize) -> u32 {
    // We unwrap here because we know that the slice has 4 bytes
    // and we know that the conversion from slice to array will not fail
    let slice: [u8; 4] = (&bytes[*offset..*offset + 4]).try_into().unwrap();

    let length = match endian {
        Endian::Big => u32::from_be_bytes(slice),
        Endian::Little => u32::from_le_bytes(slice),
    };

    *offset += 4;
    length
}
