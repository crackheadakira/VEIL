pub mod flac;
mod id3;
mod traits;

use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use crate::{
    flac::Block,
    id3::{Frame, FrameId},
};

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

    fn from_id3_frames(frames: Vec<Frame>) -> Metadata {
        let mut metadata = Metadata::default();

        for frame in frames {
            match frame {
                Frame::Duration(duration) => metadata.duration = duration,
                Frame::Text((frame_id, frame_str)) => match frame_id {
                    FrameId::Talb => metadata.album = Some(frame_str),
                    FrameId::Tit2 => metadata.name = Some(frame_str),
                    FrameId::Tpe1 => metadata.artist = Some(frame_str),
                    _ => {}
                },
                Frame::Picture(picture_data) => metadata.picture_data = Some(picture_data),
                Frame::Year(year) => metadata.year = Some(year),
                Frame::Unknown => {}
            }
        }

        metadata
    }

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

    fn read_id3<R: Read + Seek>(buffer: &'a mut Vec<u8>, reader: &mut R) -> Result<Metadata<'a>> {
        let frame_headers = id3::Id3::read_all_frames(buffer, reader)?;

        let mut id3_frames = Vec::with_capacity(frame_headers.len());
        for frame in frame_headers {
            let start = frame.data_start as usize;
            let end = start + frame.length as usize;

            let slice = &buffer[start..end];
            let block = id3::Frame::parse_by_id(frame.frame_id, slice)?;
            id3_frames.push(block);
        }

        Ok(Metadata::from_id3_frames(id3_frames))
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
            } else if ext.eq_ignore_ascii_case("mp3") {
                let file = File::open(path)?;
                let mut reader = BufReader::with_capacity(4 * 1024, file);

                let metadata = Self::read_id3(buffer, &mut reader)?;

                Ok(metadata)
            } else {
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
            SupportedFormats::ID3 => {
                let mut reader = Cursor::new(data);
                Self::read_id3(buffer, &mut reader)
            }
        }
    }

    pub fn recursive_dir(path: &Path) -> Vec<PathBuf> {
        let mut tracks = Vec::with_capacity(3_000);
        let mut stack = vec![path.to_path_buf()];

        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if let Some(ext) = path.extension().and_then(|s| s.to_str())
                        && (ext.eq_ignore_ascii_case("mp3") || ext.eq_ignore_ascii_case("flac"))
                    {
                        tracks.push(path);
                    }
                }
            }
        }

        tracks
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

#[inline(always)]
pub(crate) fn read_into_buffer_unchecked<R: Read>(
    reader: &mut R,
    buffer: &mut Vec<u8>,
    len: usize,
) -> Result<()> {
    let start_offset = buffer.len();
    let end_offset = start_offset + len;

    if end_offset > buffer.capacity() {
        buffer.reserve(end_offset - buffer.len());
    }

    debug_assert!(end_offset <= buffer.capacity());

    // SAFETY:
    // - `reserve` ensures capacity >= `end_offset`.
    // - `set_len` is thus safe because we're initializing that range immediately below.
    #[allow(unsafe_code)]
    unsafe {
        buffer.set_len(end_offset);
    };

    reader.read_exact(&mut buffer[start_offset..end_offset])?;

    Ok(())
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

    #[test]
    fn read_n_bits_u32_across_byte_boundary() {
        let bytes = [0b1010_1010, 0b1100_1100];

        // start at bit 4, read 8 bits → 1010_1100
        let val = read_n_bits_u32(&bytes, 4, 8);
        assert_eq!(val, 0b1010_1100);
    }

    #[test]
    fn metadata_from_flac_blocks_populates_fields() {
        let blocks = vec![
            Block::StreamInfo(crate::flac::StreamInfo {
                duration: 123.0,
                sample_rate: 20_000,
                total_samples: 60_000,
            }),
            Block::VorbisComment(crate::flac::VorbisComment {
                vendor_string: Some("Vendor"),
                album: Some("Album"),
                album_artist: Some("Artist"),
                title: Some("Track"),
                year: Some(2024),
                track_number: Some(3),
            }),
        ];

        let meta = Metadata::from_flac_blocks(blocks);

        assert_eq!(meta.duration, 123.0);
        assert_eq!(meta.album, Some("Album"));
        assert_eq!(meta.artist, Some("Artist"));
        assert_eq!(meta.name, Some("Track"));
        assert_eq!(meta.year, Some(2024));
        assert_eq!(meta.track_number, Some(3));
    }

    #[test]
    fn metadata_from_id3_frames_populates_fields() {
        let frames = vec![
            Frame::Text((FrameId::Tit2, "Track")),
            Frame::Text((FrameId::Tpe1, "Artist")),
            Frame::Text((FrameId::Talb, "Album")),
            Frame::Year(2023),
            Frame::Duration(180.0),
        ];

        let meta = Metadata::from_id3_frames(frames);

        assert_eq!(meta.name, Some("Track"));
        assert_eq!(meta.artist, Some("Artist"));
        assert_eq!(meta.album, Some("Album"));
        assert_eq!(meta.year, Some(2023));
        assert_eq!(meta.duration, 180.0);
    }

    #[test]
    fn invalid_flac_signature_returns_error() {
        let mut buffer = Vec::new();
        let data = b"NOTF";
        let mut reader = Cursor::new(data);

        let err = Metadata::read_flac(&mut buffer, &mut reader, true).unwrap_err();
        matches!(err, Error::InvalidFlacSignature);
    }
}
