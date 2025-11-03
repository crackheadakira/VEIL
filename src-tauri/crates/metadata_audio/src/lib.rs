mod flac;
mod id3;

use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
/// Metadata struct that holds information about an audio file
pub struct Metadata {
    /// Artist name
    pub artist: String,

    /// Album name
    pub album: String,

    /// Track name
    pub name: String,

    /// Path to the audio file
    pub file_path: String,

    /// Album type
    pub album_type: String,

    /// Duration of the album in seconds
    pub duration: f32,

    pub track_number: i32,
    /// Track number

    /// Year of publication
    pub year: u16,

    /// Picture data
    pub picture_data: Option<Rc<Vec<u8>>>,
}

pub enum SupportedFormats {
    Flac,
    ID3,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid file path")]
    InvalidFilePath,
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

pub(crate) type Result<T, U = Error> = std::result::Result<T, U>;

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
            picture_data: None,
        }
    }

    fn from_flac(file: flac::Flac) -> Metadata {
        let vorbis_comment = file.vorbis_comment;

        Metadata {
            duration: file.stream_info.duration,
            album: vorbis_comment.album.unwrap_or(String::from("Unknown")),
            artist: vorbis_comment
                .album_artist
                .unwrap_or(String::from("Unknown")),
            name: vorbis_comment.title.unwrap_or(String::from("Unknown")),
            file_path: file.file_path,
            album_type: String::from("Unknown"),
            year: vorbis_comment.year.unwrap_or(0),
            track_number: vorbis_comment.track_number.unwrap_or(-1),
            picture_data: file.picture.map(|pic| Rc::new(pic.data)),
        }
    }

    fn from_id3(file: id3::Id3) -> Metadata {
        Metadata {
            duration: -1.0,
            album: get_field_value(&file.text_frames, "TALB"),
            artist: get_field_value(&file.text_frames, "TPE1"),
            name: get_field_value(&file.text_frames, "TIT2"),
            file_path: file.file_path,
            album_type: String::from("Unknown"),
            year: get_field_value(&file.text_frames, "TYER")
                .parse()
                .unwrap_or(0),
            track_number: get_field_value(&file.text_frames, "TRCK")
                .split('/')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1),
            picture_data: file.attached_picture.map(|pic| Rc::new(pic.picture_data)),
        }
    }

    /// Create a `Metadata` struct from a valid audio file
    pub fn from_file(path: &std::path::Path, skip_picture: bool) -> Result<Metadata> {
        if let Some(os_ext) = path.extension()
            && let Some(ext) = os_ext.to_str()
        {
            match ext {
                "flac" => {
                    let file = flac::Flac::new(path, skip_picture)?;
                    Ok(Metadata::from_flac(file))
                }
                "mp3" => {
                    let file = id3::Id3::new(path)?;
                    Ok(Metadata::from_id3(file))
                }
                _ => Err(Error::UnsupportedFileType),
            }
        } else {
            Err(Error::InvalidFilePath)
        }
    }

    pub fn from_bytes(
        data: &[u8],
        format: SupportedFormats,
        skip_picture: bool,
    ) -> Result<Metadata> {
        match format {
            SupportedFormats::Flac => {
                let file = flac::Flac::from_bytes(data, skip_picture)?;
                Ok(Metadata::from_flac(file))
            }
            SupportedFormats::ID3 => Err(Error::UnsupportedId3Version),
        }
    }

    /// Create a vec of `Metadata` structs from a list of audio files
    pub fn from_files(
        file_paths: &[std::path::PathBuf],
        skip_picture: bool,
    ) -> Result<Vec<Metadata>> {
        let mut all_metadata = Vec::with_capacity(file_paths.len());

        for path in file_paths {
            let metadata = Metadata::from_file(path, skip_picture);
            match metadata {
                Ok(m) => all_metadata.push(m),
                Err(e) => {
                    logging::warn!("Skipping over audio file ({path:?}) due to error: {e}");
                    continue;
                }
            }
        }
        Ok(all_metadata)
    }
}

// Helper functions

/// Get value at key from hashmap, if key doesn't exist return `Unknown`
fn get_field_value(fields: &HashMap<String, String>, key: &str) -> String {
    fields
        .get(key)
        .unwrap_or(&String::from("Unknown"))
        .to_string()
}

#[inline(always)]
/// Read `n` bits from a byte slice starting at a given bit position
fn read_n_bits<T>(bytes: &[u8], start_bit: usize, n_bits: usize) -> T
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

/// Endian enum
enum Endian {
    /// Big endian
    Big,
    /// Little endian
    Little,
}

#[inline(always)]
/// Convert a slice of bytes to a u32 integer
fn u32_from_bytes(endian: Endian, bytes: &[u8], offset: &mut usize) -> u32 {
    let b0 = bytes[*offset] as u32;
    let b1 = bytes[*offset + 1] as u32;
    let b2 = bytes[*offset + 2] as u32;
    let b3 = bytes[*offset + 3] as u32;
    *offset += 4;

    match endian {
        Endian::Big => (b0 << 24) | (b1 << 16) | (b2 << 8) | b3,
        Endian::Little => (b3 << 24) | (b2 << 16) | (b1 << 8) | b0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_from_bytes_little_endian() {
        let result = u32_from_bytes(Endian::Little, &[0x00, 0x00, 0x00, 0x00], &mut 0);
        assert_eq!(result, 0x0000);

        let result = u32_from_bytes(Endian::Little, &[0x01, 0x02, 0x03, 0x04], &mut 0);
        assert_eq!(result, 0x04030201);

        let result = u32_from_bytes(Endian::Little, &[0x4, 0x3, 0x2, 0x1], &mut 0);
        assert_eq!(result, 0x01020304);

        let result = u32_from_bytes(Endian::Little, &[0x10, 0x20, 0x30, 0x40], &mut 0);
        assert_eq!(result, 0x40302010);

        let result = u32_from_bytes(Endian::Little, &[0x10, 0x20, 0x40, 0x30], &mut 0);
        assert_eq!(result, 0x30402010);
    }

    #[test]
    fn u32_from_bytes_big_endian() {
        let result = u32_from_bytes(Endian::Big, &[0x00, 0x00, 0x00, 0x00], &mut 0);
        assert_eq!(result, 0x0000);

        let result = u32_from_bytes(Endian::Big, &[0x01, 0x02, 0x03, 0x04], &mut 0);
        assert_eq!(result, 0x01020304);

        let result = u32_from_bytes(Endian::Big, &[0x4, 0x3, 0x2, 0x1], &mut 0);
        assert_eq!(result, 0x04030201);

        let result = u32_from_bytes(Endian::Big, &[0x10, 0x20, 0x30, 0x40], &mut 0);
        assert_eq!(result, 0x10203040);

        let result = u32_from_bytes(Endian::Big, &[0x10, 0x20, 0x40, 0x30], &mut 0);
        assert_eq!(result, 0x10204030);
    }
}
