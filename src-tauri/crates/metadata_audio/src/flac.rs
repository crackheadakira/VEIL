use std::io::{Read, Seek};

use crate::{
    Error, Result, read_into_buffer_unchecked, read_n_bits_u32, read_n_bits_u64, u32_from_bytes_be,
    u32_from_bytes_le,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum BlockType {
    StreamInfo = 0,
    VorbisComment = 4,
    Picture = 6,
    Unknown,
}

impl BlockType {
    fn from_u8(id: u8) -> BlockType {
        match id {
            0 => BlockType::StreamInfo,
            4 => BlockType::VorbisComment,
            6 => BlockType::Picture,
            _ => BlockType::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum Block<'a> {
    VorbisComment(VorbisComment<'a>),
    StreamInfo(StreamInfo),
    Picture(Picture<'a>),
    Unknown,
}

#[derive(Debug)]
pub struct BlockHeader {
    pub is_last: bool,
    pub block_type: BlockType,
    pub start: u32,
    pub length: u32,
}

impl<'a> Block<'a> {
    pub fn parse_by_block_type(block_type: BlockType, data: &'a [u8]) -> Result<Block<'a>> {
        match block_type {
            BlockType::StreamInfo => Ok(Block::StreamInfo(StreamInfo::from_bytes(data))),
            BlockType::VorbisComment => Ok(Block::VorbisComment(VorbisComment::from_bytes(data)?)),
            BlockType::Picture => Ok(Block::Picture(Picture::from_bytes(data))),
            BlockType::Unknown => Ok(Block::Unknown),
        }
    }

    pub fn parse_block_header<R: Read + Seek>(reader: &mut R) -> Result<BlockHeader> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;

        // Big-endian, first bit is whether it's last,
        // the remaining bits state the metadata block type
        let header_byte = buf[0];
        let is_last = (header_byte & 0x80) != 0;
        let block_type = BlockType::from_u8(header_byte & 0x7F);

        // Length is max 3 bytes --> 16MB
        let length = u32::from_be_bytes([0x00, buf[1], buf[2], buf[3]]);

        Ok(BlockHeader {
            is_last,
            block_type,
            start: 0,
            length,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub duration: f32,
    pub sample_rate: u32,
    pub total_samples: u64,
}

impl Default for StreamInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamInfo {
    pub fn new() -> StreamInfo {
        StreamInfo {
            duration: -1.0,
            sample_rate: 0,
            total_samples: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> StreamInfo {
        let mut stream_info = StreamInfo::new();
        let mut i = 0;

        // stream_info.min_block_size = u16::from_be_bytes((&bytes[i..i + 2]).try_into().unwrap());
        i += 2;

        // stream_info.max_block_size = u16::from_be_bytes((&bytes[i..i + 2]).try_into().unwrap());
        i += 2;

        // stream_info.min_frame_size = read_n_bits(&bytes[i..i + 3], 0, 24);
        i += 3;

        // stream_info.max_frame_size = read_n_bits(&bytes[i..i + 3], 0, 24);
        i += 3;

        // Read the sample rate (20 bits)
        stream_info.sample_rate = read_n_bits_u32(&bytes[i..i + 3], 0, 20);
        i += 2; // But only increment by 2 bytes

        // We need to skip the first 4 bits to reach where sample rate ended
        // stream_info.channels = read_n_bits(&bytes[i..i + 1], 4, 3);
        // stream_info.bits_per_sample = read_n_bits(&bytes[i..i + 2], 7, 5);
        i += 1;

        // from last one we're at bit 24, but bits_per_sample reached until 28 so we need to skip 4 bits
        // total_samples is 36 bits long which is 4.5 bytes, so we need to read 5 bytes
        stream_info.total_samples = read_n_bits_u64(&bytes[i..i + 5], 4, 36);

        stream_info.duration = stream_info.total_samples as f32 / stream_info.sample_rate as f32;

        stream_info
    }
}

#[derive(Debug, Clone, Default)]
pub struct VorbisComment<'a> {
    pub vendor_string: Option<&'a str>,

    pub album: Option<&'a str>,

    pub album_artist: Option<&'a str>,

    pub title: Option<&'a str>,

    pub year: Option<u16>,

    pub track_number: Option<u32>,
}

impl<'a> VorbisComment<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<VorbisComment<'a>> {
        let mut vorbis = VorbisComment::default();
        let mut i = 0;

        let vendor_length = u32_from_bytes_le(bytes, &mut i) as usize;

        let vendor_string = std::str::from_utf8(&bytes[i..i + vendor_length])?;
        vorbis.vendor_string = Some(vendor_string);

        i += vendor_length;

        let num_comments = u32_from_bytes_le(bytes, &mut i);
        for _ in 0..num_comments {
            let comment_length = u32_from_bytes_le(bytes, &mut i) as usize;
            let comment_slice = &bytes[i..i + comment_length];

            if let Some(eq_pos) = comment_slice.iter().position(|&b| b == b'=') {
                let (key, value) = comment_slice.split_at(eq_pos);

                let value_raw = &value[1..];
                let value = std::str::from_utf8(value_raw)?.trim();

                match key {
                    b"ALBUM" => vorbis.album = Some(value),
                    b"ALBUMARTIST" => vorbis.album_artist = Some(value),
                    b"TITLE" => vorbis.title = Some(value),
                    b"YEAR" => vorbis.year = Self::parse_u16_ascii(value_raw),
                    b"TRACKNUMBER" => vorbis.track_number = Self::parse_u32_ascii(value_raw),
                    _ => {}
                }
            }

            i += comment_length;
        }

        Ok(vorbis)
    }

    #[inline(always)]
    fn parse_u16_ascii(bytes: &[u8]) -> Option<u16> {
        let mut n = 0u16;
        for &b in bytes {
            if !b.is_ascii_digit() {
                return None;
            }
            n = n * 10 + (b - b'0') as u16;
        }
        Some(n)
    }

    #[inline(always)]
    fn parse_u32_ascii(bytes: &[u8]) -> Option<u32> {
        if bytes.is_empty() {
            return None;
        }

        let mut n = 0u32;
        for &b in bytes {
            if !b.is_ascii_digit() {
                return None;
            }
            n = n * 10 + (b - b'0') as u32;
        }
        Some(n)
    }
}

#[derive(Debug, Clone)]
pub struct Picture<'a> {
    // pub picture_type: u32,
    // pub mime_type: String,
    // pub description: String,
    // pub width: u32,
    // pub height: u32,
    // pub color_depth: u32,
    // pub indexed_color: u32,
    pub data: &'a [u8],
}

impl<'a> Picture<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Picture<'a> {
        let mut i = 0;

        // picture.picture_type = u32_from_bytes(Endian::Big, &bytes, &mut i);
        i += 4;

        let mime_length = u32_from_bytes_be(bytes, &mut i) as usize;

        // picture.mime_type = String::from_utf8_lossy(&bytes[i..i + mime_length]).to_string();
        i += mime_length;

        let description_length = u32_from_bytes_be(bytes, &mut i) as usize;

        // picture.description =
        //     String::from_utf8_lossy(&bytes[i..i + description_length]).to_string();
        i += description_length;

        // picture.width = u32_from_bytes(Endian::Big, &bytes, &mut i);

        // picture.height = u32_from_bytes(Endian::Big, &bytes, &mut i);

        // picture.color_depth = u32_from_bytes(Endian::Big, &bytes, &mut i);

        // picture.indexed_color = u32_from_bytes(Endian::Big, &bytes, &mut i);

        i += 16;

        let picture_length = u32_from_bytes_be(bytes, &mut i) as usize;

        Picture {
            data: &bytes[i..i + picture_length],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Flac<'a> {
    pub stream_info: StreamInfo,
    pub picture: Option<Picture<'a>>,
    pub vorbis_comment: VorbisComment<'a>,
}

impl<'a> Flac<'a> {
    pub fn read_all_blocks<R: Read + Seek>(
        block_buffer: &mut Vec<u8>,
        reader: &mut R,
        skip_picture: bool,
    ) -> Result<Vec<BlockHeader>> {
        // Check the FLAC signature (fLaC)
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;
        if &signature != b"fLaC" {
            return Err(Error::InvalidFlacSignature);
        }

        let mut stream_info_found = false;
        let mut vorbis_comment_found = false;
        let mut picture_found = false;

        let mut headers = Vec::with_capacity(3);
        loop {
            let block_header = Block::parse_block_header(reader)?;
            let is_last = block_header.is_last;

            let read_block = block_header.block_type != BlockType::Unknown
                && !(skip_picture && block_header.block_type == BlockType::Picture);

            let start = block_buffer.len() as u32;
            if read_block {
                read_into_buffer_unchecked(reader, block_buffer, block_header.length as usize)?;

                headers.push(BlockHeader {
                    start,
                    length: block_header.length,
                    block_type: block_header.block_type,
                    is_last,
                });

                match block_header.block_type {
                    BlockType::StreamInfo => stream_info_found = true,
                    BlockType::VorbisComment => vorbis_comment_found = true,
                    BlockType::Picture => picture_found = true,
                    BlockType::Unknown => (),
                }
            } else {
                reader.seek_relative(block_header.length as i64)?;
            }

            if is_last
                || (stream_info_found && vorbis_comment_found && (picture_found || skip_picture))
            {
                break;
            }
        }

        Ok(headers)
    }
}
