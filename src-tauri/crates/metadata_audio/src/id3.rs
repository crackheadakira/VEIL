use std::io::{Read, Seek};

use crate::{Error, Result, read_into_buffer_unchecked};

pub struct Id3 {}

impl Id3 {
    pub fn read_all_frames<R: Read + Seek>(
        block_buffer: &mut Vec<u8>,
        reader: &mut R,
    ) -> Result<Vec<FrameHeader>> {
        let _header = Id3Header::parse(reader)?;

        // TODO: Depending on major & minor versions, handle different methods of parsing IF there are
        // changes to the metadata parsing, still have to research that.

        let mut frame_headers = Vec::with_capacity(6);
        let mut found = FoundFrames::default();
        while let Some(mut header) = FrameHeader::parse(reader)? {
            let start = block_buffer.len();
            read_into_buffer_unchecked(reader, block_buffer, header.length as usize)?;

            header.data_start = start as u32;

            match header.frame_id {
                FrameId::Tit2 => found.title = true,
                FrameId::Tpe1 => found.artist = true,
                FrameId::Talb => found.album = true,
                FrameId::Tyer => found.year = true,
                FrameId::Time => found.duration = true,
                FrameId::Apic => found.picture = true,
                FrameId::Unknown => {}
            }
            if header.frame_id != FrameId::Unknown {
                frame_headers.push(header);
            }

            if found.all() {
                break;
            }
        }

        Ok(frame_headers)
    }
}

#[derive(Default)]
struct FoundFrames {
    title: bool,
    artist: bool,
    album: bool,
    year: bool,
    duration: bool,
    picture: bool,
}

impl FoundFrames {
    fn all(&self) -> bool {
        self.title && self.artist && self.album && self.year && self.duration && self.picture
    }
}

#[derive(Debug)]
struct Id3Header {
    major_version: u8,
    minor_version: u8,
    tag_size: u32,
}

impl Id3Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let mut header_bytes = [0u8; 10];
        reader.read_exact(&mut header_bytes)?;

        let mut header = Self {
            major_version: 0,
            minor_version: 0,
            tag_size: 0,
        };

        if &header_bytes[0..3] != b"ID3" {
            return Err(Error::InvalidId3Signature);
        };

        header.major_version = header_bytes[3];
        header.minor_version = header_bytes[4];

        let size_bytes: &[u8] = &header_bytes[6..10];

        header.tag_size = size_bytes
            .iter()
            .fold(0u32, |acc, &b| (acc << 7) | u32::from(b & 0x7F));

        // Skip past the extended header for simplicity.
        let has_extended_header = &header_bytes[5] & 0x40 != 0;
        if has_extended_header {
            let mut size_bytes = [0u8; 4];
            reader.read_exact(&mut size_bytes)?;
            let ext_size = u32::from_be_bytes(size_bytes);

            reader.seek_relative((ext_size - 4) as i64)?;
        }

        Ok(header)
    }
}

pub struct FrameHeader {
    pub frame_id: FrameId,
    pub data_start: u32,
    pub length: u32,
}

impl FrameHeader {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Option<Self>> {
        let mut header_bytes = [0u8; 10];
        reader.read_exact(&mut header_bytes)?;
        let start = reader.stream_position()?;

        // Padding bytes have been reached, so return None for early loop termination
        if header_bytes[0..4].iter().all(|&b| b == 0) {
            return Ok(None);
        }

        let frame_id = FrameId::from_bytes(&header_bytes[0..4]);

        // seems to be only for ID3v2.3
        let frame_size = u32::from_be_bytes(
            header_bytes[4..8]
                .try_into()
                .expect("slice should have length 4"),
        );

        Ok(Some(Self {
            data_start: start as u32,
            frame_id,
            length: frame_size,
        }))
    }
}

pub enum Frame<'a> {
    Picture(&'a [u8]),
    Text((FrameId, &'a str)),
    Duration(f32),
    Year(u16),
    Unknown,
}

impl<'a> Frame<'a> {
    pub fn parse_by_id(frame_id: FrameId, data: &'a [u8]) -> Result<Self> {
        let parsed_frame = match frame_id {
            FrameId::Tit2 | FrameId::Tpe1 | FrameId::Talb => {
                // TODO: check if worth doing unsafe in benchmarks
                let text_data = str::from_utf8(&data[1..])?.trim();
                Self::Text((frame_id, text_data))
            }
            FrameId::Time => {
                let bytes = &data[1..5]; // HHMM
                let hours = (bytes[0] - b'0') as u32 * 10 + (bytes[1] - b'0') as u32;
                let minutes = (bytes[2] - b'0') as u32 * 10 + (bytes[3] - b'0') as u32;
                Self::Duration((hours * 3600 + minutes * 60) as f32)
            }
            FrameId::Tyer => {
                let bytes = &data[1..5]; // YYYY
                let year = (bytes[0] - b'0') as u16 * 1000
                    + (bytes[1] - b'0') as u16 * 100
                    + (bytes[2] - b'0') as u16 * 10
                    + (bytes[3] - b'0') as u16;
                Self::Year(year)
            }
            FrameId::Apic => Self::Picture(&data[1..]),
            FrameId::Unknown => Self::Unknown,
        };

        Ok(parsed_frame)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum FrameId {
    Tit2,
    Tpe1,
    Talb,
    Apic,
    Time,
    Tyer,
    Unknown,
}

impl FrameId {
    fn from_bytes(bytes: &[u8]) -> FrameId {
        match bytes {
            b"TIT2" => FrameId::Tit2,
            b"TPE1" => FrameId::Tpe1,
            b"TALB" => FrameId::Talb,
            b"APIC" => FrameId::Apic,
            b"TYER" => FrameId::Tyer,
            b"TIME" => FrameId::Time,
            _ => FrameId::Unknown,
        }
    }
}
