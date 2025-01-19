use byteorder::{ReadBytesExt, BE};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::error;

#[derive(Clone, Copy, Debug)]
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
pub enum Block {
    VorbisComment(VorbisComment),
    StreamInfo(StreamInfo),
    Picture(Picture),
    Unknown,
}

impl Block {
    pub fn read_from(reader: &mut dyn Read) -> error::Result<(bool, u32, Block)> {
        let byte = reader.read_u8()?;
        let is_last = (byte & 0x80) != 0;
        let block_type = BlockType::from_u8(byte & 0x7F);
        let length = reader.read_uint::<BE>(3)?;

        let mut data = Vec::new();
        reader.take(length).read_to_end(&mut data).unwrap();

        let block = match block_type {
            BlockType::StreamInfo => Block::StreamInfo(StreamInfo::from_bytes(data)),
            BlockType::VorbisComment => Block::VorbisComment(VorbisComment::from_bytes(data)),
            BlockType::Picture => Block::Picture(Picture::from_bytes(data)),
            BlockType::Unknown => Block::Unknown,
        };

        Ok((is_last, (length as u32) + 4, block))
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
            duration: 0.0,
            sample_rate: 0,
            total_samples: 0,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> StreamInfo {
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
        stream_info.sample_rate = read_n_bits(&bytes[i..i + 3], 0, 20);
        i += 2; // But only increment by 2 bytes

        // We need to skip the first 4 bits to reach where sample rate ended
        // stream_info.channels = read_n_bits(&bytes[i..i + 1], 4, 3);
        // stream_info.bits_per_sample = read_n_bits(&bytes[i..i + 2], 7, 5);
        i += 1;

        // from last one we're at bit 24, but bits_per_sample reached until 28 so we need to skip 4 bits
        // total_samples is 36 bits long which is 4.5 bytes, so we need to read 5 bytes
        stream_info.total_samples = read_n_bits(&bytes[i..i + 5], 4, 36);

        stream_info.duration = stream_info.total_samples as f32 / stream_info.sample_rate as f32;

        stream_info
    }
}

#[derive(Debug, Clone)]
pub struct VorbisComment {
    pub vendor_string: String,
    pub fields: HashMap<String, Vec<String>>,
}

impl Default for VorbisComment {
    fn default() -> Self {
        Self::new()
    }
}

impl VorbisComment {
    pub fn new() -> VorbisComment {
        VorbisComment {
            vendor_string: String::new(),
            fields: HashMap::new(),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> VorbisComment {
        let mut vorbis = VorbisComment::new();
        let mut i = 0;

        let vendor_length = u32::from_le_bytes((&bytes[i..i + 4]).try_into().unwrap()) as usize;
        i += 4;

        vorbis.vendor_string = String::from_utf8_lossy(&bytes[i..i + vendor_length]).to_string();
        i += vendor_length;

        let num_comments = u32::from_le_bytes((&bytes[i..i + 4]).try_into().unwrap());
        i += 4;

        for _ in 0..num_comments {
            let comment_length =
                u32::from_le_bytes((&bytes[i..i + 4]).try_into().unwrap()) as usize;
            i += 4;

            let comments = String::from_utf8_lossy(&bytes[i..i + comment_length]).to_string();
            i += comment_length;

            let comments_split: Vec<&str> = comments.splitn(2, '=').collect();
            let key = comments_split[0].to_ascii_uppercase();
            let value = comments_split[1].to_owned();

            vorbis
                .fields
                .entry(key)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(value);
        }

        vorbis
    }
}

#[derive(Debug, Clone)]
pub struct Picture {
    pub picture_type: u32,
    pub mime_type: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub color_depth: u32,
    pub indexed_color: u32,
    pub data: Vec<u8>,
}

impl Default for Picture {
    fn default() -> Self {
        Self::new()
    }
}

impl Picture {
    pub fn new() -> Picture {
        Picture {
            picture_type: 0,
            mime_type: String::new(),
            description: String::new(),
            width: 0,
            height: 0,
            color_depth: 0,
            indexed_color: 0,
            data: Vec::new(),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Picture {
        let mut picture = Picture::new();
        let mut i = 0;

        picture.picture_type = read_u32_from_bytes(&bytes, &mut i);

        let mime_length = read_u32_from_bytes(&bytes, &mut i) as usize;

        picture.mime_type = String::from_utf8_lossy(&bytes[i..i + mime_length]).to_string();
        i += mime_length;

        let description_length = read_u32_from_bytes(&bytes, &mut i) as usize;

        picture.description =
            String::from_utf8_lossy(&bytes[i..i + description_length]).to_string();
        i += description_length;

        picture.width = read_u32_from_bytes(&bytes, &mut i);

        picture.height = read_u32_from_bytes(&bytes, &mut i);

        picture.color_depth = read_u32_from_bytes(&bytes, &mut i);

        picture.indexed_color = read_u32_from_bytes(&bytes, &mut i);

        let picture_length = read_u32_from_bytes(&bytes, &mut i) as usize;

        picture.data = bytes[i..i + picture_length].to_vec();

        picture
    }
}

#[derive(Debug, Clone)]
pub struct Flac {
    pub file_path: String,
    pub stream_info: StreamInfo,
    pub vorbis_comment: Option<VorbisComment>,
    pub picture: Option<Picture>,
}

impl Flac {
    pub fn new(file_path: &Path) -> error::Result<Flac> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);

        // Check the FLAC signature (fLaC)
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;
        if &signature != b"fLaC" {
            return Err("Invalid FLAC signature".into());
        }
        let mut stream_info = StreamInfo::new();
        let mut vorbis_comment = None;
        let mut picture = None;

        loop {
            let result = Block::read_from(&mut reader)?;
            let (flag, _, block) = result;

            match block {
                Block::StreamInfo(si) => {
                    stream_info = si;
                }
                Block::VorbisComment(vc) => {
                    vorbis_comment = Some(vc);
                }
                Block::Picture(pic) => {
                    picture = Some(pic);
                }
                Block::Unknown => {}
            }

            if flag
                || (vorbis_comment.is_some() && picture.is_some() && stream_info.total_samples > 0)
            {
                break;
            }
        }

        Ok(Flac {
            file_path: file_path.to_string_lossy().to_string(),
            stream_info,
            vorbis_comment,
            picture,
        })
    }
}

fn read_u32_from_bytes(bytes: &[u8], offset: &mut usize) -> u32 {
    let length = u32::from_be_bytes((&bytes[*offset..*offset + 4]).try_into().unwrap());
    *offset += 4;
    length
}

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
