use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::{u32_from_bytes, Endian, MetadataError};

pub enum FrameType {
    AttachedPicture,
    Text(String),
    Unknown,
}

impl FrameType {
    fn from_str(id: &str) -> Self {
        match id {
            "APIC" => Self::AttachedPicture,
            id if id.starts_with('T') => Self::Text(String::from(id)),
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum Frame {
    Text(TextFrame),
    AttachedPicture(AttachedPicture),
    Unknown,
}

impl Frame {
    pub fn read_from(reader: &mut dyn Read) -> Result<(u32, Frame), MetadataError> {
        let mut overview = [0u8; 10];
        reader.read_exact(&mut overview)?;

        let frame_type_str = std::str::from_utf8(&overview[0..4])?;
        let frame_type = FrameType::from_str(frame_type_str);
        let size = u32_from_bytes(Endian::Big, &overview[4..8], &mut 0_usize);

        let mut data = Vec::new();
        reader.take(size as u64).read_to_end(&mut data)?;

        let frame = match frame_type {
            FrameType::Text(string_type) => Frame::Text(TextFrame::from_bytes(data, string_type)?),
            FrameType::AttachedPicture => {
                Frame::AttachedPicture(AttachedPicture::from_bytes(data)?)
            }
            _ => Self::Unknown,
        };

        Ok((size, frame))
    }
}

#[derive(Debug)]
pub struct TextFrame {
    text_type: String,
    string_value: String,
}

impl TextFrame {
    pub fn new() -> Self {
        Self {
            text_type: String::from(""),
            string_value: String::from(""),
        }
    }

    pub fn from_bytes(data: Vec<u8>, string_type: String) -> Result<Self, MetadataError> {
        let mut text_frame = TextFrame::new();
        text_frame.text_type = string_type;

        text_frame.string_value = String::from_utf8(data[1..].to_vec())?;

        Ok(text_frame)
    }
}

#[derive(Debug)]
pub struct AttachedPicture {
    pub picture_data: Vec<u8>,
}

impl Default for AttachedPicture {
    fn default() -> Self {
        Self::new()
    }
}

impl AttachedPicture {
    fn new() -> Self {
        Self {
            picture_data: Vec::new(),
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self, MetadataError> {
        let mut ap = Self::new();
        let single_terminator = (data[0] != 0 || data[0] != 3) as usize;
        let mut i = 1; // skip text encoding

        let mime_type_end = data[i..].iter().position(|&x| x == 0).unwrap() + single_terminator;
        i += mime_type_end;

        i += 1;

        let description_end = data[i..].iter().position(|&x| x == 0).unwrap() + single_terminator;
        i += description_end;

        ap.picture_data = data[i..].to_vec();

        Ok(ap)
    }
}

pub struct Id3 {
    pub file_path: String,
    pub text_frames: HashMap<String, String>,
    pub attached_picture: Option<AttachedPicture>,
}

impl Id3 {
    pub fn new(file_path: &Path) -> Result<Self, MetadataError> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);

        // Check the header
        let mut header = [0u8; 10];
        reader.read_exact(&mut header)?;
        if &header[0..3] != b"ID3" {
            return Err(MetadataError::InvalidId3Signature);
        }
        if header[3] != 3 {
            return Err(MetadataError::UnsupportedId3Version);
        }

        let size_bytes: &[u8] = &header[6..10];
        let total_id3_size = ((size_bytes[0] as usize & 0x7F) << 21)
            | ((size_bytes[1] as usize & 0x7F) << 14)
            | ((size_bytes[2] as usize & 0x7F) << 7)
            | (size_bytes[3] as usize & 0x7F);

        let has_extended_header = &header[6] & 0x40 != 0;
        if has_extended_header {
            // just skip past the extended_header
            reader.read_exact(&mut [0u8; 10])?;
        }

        let mut text_frames: HashMap<String, String> = HashMap::new();
        let mut attached_picture = None;

        let mut total_read = 0;
        loop {
            let (frame_size, result) = Frame::read_from(&mut reader)?;
            total_read += frame_size as usize;
            match result {
                Frame::Text(tf) => {
                    text_frames.insert(tf.text_type, tf.string_value);
                }
                Frame::AttachedPicture(ap) => attached_picture = Some(ap),
                Frame::Unknown => (),
            };

            if total_read >= total_id3_size
                || (text_frames.contains_key("TIT2")
                    && text_frames.contains_key("TRCK")
                    && text_frames.contains_key("TYER")
                    && text_frames.contains_key("TPE1")
                    && text_frames.contains_key("TALB")
                    && attached_picture.is_some())
            {
                break;
            }
        }

        Ok(Id3 {
            file_path: file_path.to_string_lossy().into_owned(),
            text_frames,
            attached_picture,
        })
    }
}
