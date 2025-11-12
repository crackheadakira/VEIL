pub mod flac;
mod id3;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use crate::flac::Block;

#[derive(Debug, Clone, Default)]
/// Metadata struct that holds information about an audio file
pub struct Metadata<'a> {
    /// Artist name
    pub artist: Option<&'a str>,

    /// Album name
    pub album: Option<&'a str>,

    /// Track name
    pub name: Option<&'a str>,

    /// Duration of the album in seconds
    pub duration: f32,

    /// Track number
    pub track_number: Option<u32>,

    /// Year of publication
    pub year: Option<u16>,

    /// Picture data
    pub picture_data: Option<&'a [u8]>,
}

pub enum SupportedFormats {
    Flac,
    ID3,
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

impl<'a> Metadata<'a> {
    fn from_flac_blocks(blocks: Vec<Block>) -> Metadata {
        let mut metadata = Metadata::default();

        for block in blocks {
            match block {
                Block::StreamInfo(stream_info) => {
                    metadata.duration = stream_info.duration;
                }
                Block::VorbisComment(vorbis_comment) => {
                    metadata.album = vorbis_comment.album;
                    metadata.artist = vorbis_comment.album_artist;
                    metadata.name = vorbis_comment.title;
                    metadata.year = vorbis_comment.year;
                    metadata.track_number = vorbis_comment.track_number;
                }
                Block::Picture(picture) => {
                    metadata.picture_data = Some(picture.data);
                }
                Block::Unknown => {}
            }
        }

        metadata
    }

    /*fn from_id3(file: id3::Id3) -> Metadata {
        Metadata {
            duration: -1.0,
            album: get_field_value(&file.text_frames, "TALB"),
            artist: get_field_value(&file.text_frames, "TPE1"),
            name: get_field_value(&file.text_frames, "TIT2"),
            file_path: file.file_path,
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
    }*/

    fn read_flac<R: Read + Seek>(
        buffer: &'a mut Vec<u8>,
        reader: &mut R,
        skip_picture: bool,
    ) -> Result<Metadata<'a>> {
        let block_headers = flac::Flac::read_all_blocks(buffer, reader, skip_picture)?;

        let mut flac_blocks = Vec::with_capacity(block_headers.len());
        for header in block_headers {
            let start = header.start as usize;
            let end = (header.start + header.length) as usize;

            let slice = &buffer[start..end];
            let block = flac::Block::parse_by_block_type(header.block_type, slice)?;
            flac_blocks.push(block);
        }

        Ok(Metadata::from_flac_blocks(flac_blocks))
    }

    /// Create a `Metadata` struct from a valid audio file
    pub fn from_file(
        buffer: &'a mut Vec<u8>,
        path: &'a Path,
        skip_picture: bool,
    ) -> Result<Metadata<'a>> {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext.eq_ignore_ascii_case("flac") {
                let file = File::open(path)?;
                let mut reader = BufReader::with_capacity(4 * 1024, file);

                let metadata = Self::read_flac(buffer, &mut reader, skip_picture)?;

                Ok(metadata)
            }
            /*"mp3" => {
                let file = id3::Id3::new(path)?;
                Ok(Metadata::from_id3(file))
            }*/
            else {
                Err(Error::UnsupportedFileType)
            }
        } else {
            Err(Error::InvalidFilePath)
        }
    }

    pub fn from_bytes(
        buffer: &'a mut Vec<u8>,
        data: &[u8],
        format: SupportedFormats,
        skip_picture: bool,
    ) -> Result<Metadata<'a>> {
        match format {
            SupportedFormats::Flac => {
                let mut reader = Cursor::new(data);
                Self::read_flac(buffer, &mut reader, skip_picture)
            }
            SupportedFormats::ID3 => Err(Error::UnsupportedFileType),
        }
    }

    pub fn recursive_dir(path: &Path) -> Vec<std::path::PathBuf> {
        let paths = std::fs::read_dir(path).unwrap();
        let mut tracks = Vec::new();

        for path in paths {
            let path = path.unwrap().path();
            if path.is_dir() {
                tracks.extend(Self::recursive_dir(&path));
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

    /// The resulting output is needed by [`Metadata::from_files_smart`].
    pub fn collect_album_files_for_smart(path: &Path) -> Result<Vec<Vec<PathBuf>>> {
        let mut albums: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
        Self::collect_recursive(path, &mut albums)?;
        Ok(albums.into_values().collect())
    }

    fn collect_recursive(dir: &Path, albums: &mut HashMap<PathBuf, Vec<PathBuf>>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_recursive(&path, albums)?;
                continue;
            }

            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if !ext.eq_ignore_ascii_case("mp3") && !ext.eq_ignore_ascii_case("flac") {
                continue;
            }

            if let Some(parent) = path.parent() {
                albums.entry(parent.to_path_buf()).or_default().push(path);
            }
        }

        Ok(())
    }
}

#[inline(always)]
/// Read `n` bits from a byte slice starting at a given bit position
fn read_n_bits_u32(bytes: &[u8], start_bit: usize, n_bits: usize) -> u32 {
    debug_assert!(n_bits <= 32);

    let byte_offset = start_bit / 8;
    let bit_offset = start_bit % 8;

    // Load 4 bytes (u32) from the slice
    let mut buf = [0u8; 4];
    let len = bytes.len() - byte_offset;
    buf[..len.min(4)].copy_from_slice(&bytes[byte_offset..byte_offset + len.min(4)]);

    let val = u32::from_be_bytes(buf);
    let shift = 32 - n_bits - bit_offset;
    (val >> shift) & ((1 << n_bits) - 1)
}

#[inline(always)]
/// Read `n` bits from a byte slice starting at a given bit position
fn read_n_bits_u64(bytes: &[u8], start_bit: usize, n_bits: usize) -> u64 {
    debug_assert!(n_bits <= 64);

    let byte_offset = start_bit / 8;
    let bit_offset = start_bit % 8;

    // Load 8 bytes (u64) from the slice
    let mut buf = [0u8; 8];
    let len = bytes.len() - byte_offset;
    buf[..len.min(8)].copy_from_slice(&bytes[byte_offset..byte_offset + len.min(8)]);

    let val = u64::from_be_bytes(buf);
    let shift = 64 - n_bits - bit_offset;
    (val >> shift) & ((1u64 << n_bits) - 1)
}

/// Convert a little-endian slice of bytes to a `u32` integer
#[inline(always)]
fn u32_from_bytes_le(bytes: &[u8], offset: &mut usize) -> u32 {
    let res = u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    res
}

/// Convert a big-endian slice of bytes to a `u32` integer
#[inline(always)]
fn u32_from_bytes_be(bytes: &[u8], offset: &mut usize) -> u32 {
    let res = u32::from_be_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_from_bytes_little_endian() {
        let result = u32_from_bytes_le(&[0x00, 0x00, 0x00, 0x00], &mut 0);
        assert_eq!(result, 0x0000);

        let result = u32_from_bytes_le(&[0x01, 0x02, 0x03, 0x04], &mut 0);
        assert_eq!(result, 0x04030201);

        let result = u32_from_bytes_le(&[0x4, 0x3, 0x2, 0x1], &mut 0);
        assert_eq!(result, 0x01020304);

        let result = u32_from_bytes_le(&[0x10, 0x20, 0x30, 0x40], &mut 0);
        assert_eq!(result, 0x40302010);

        let result = u32_from_bytes_le(&[0x10, 0x20, 0x40, 0x30], &mut 0);
        assert_eq!(result, 0x30402010);
    }

    #[test]
    fn u32_from_bytes_big_endian() {
        let result = u32_from_bytes_be(&[0x00, 0x00, 0x00, 0x00], &mut 0);
        assert_eq!(result, 0x0000);

        let result = u32_from_bytes_be(&[0x01, 0x02, 0x03, 0x04], &mut 0);
        assert_eq!(result, 0x01020304);

        let result = u32_from_bytes_be(&[0x4, 0x3, 0x2, 0x1], &mut 0);
        assert_eq!(result, 0x04030201);

        let result = u32_from_bytes_be(&[0x10, 0x20, 0x30, 0x40], &mut 0);
        assert_eq!(result, 0x10203040);

        let result = u32_from_bytes_be(&[0x10, 0x20, 0x40, 0x30], &mut 0);
        assert_eq!(result, 0x10204030);
    }
}
