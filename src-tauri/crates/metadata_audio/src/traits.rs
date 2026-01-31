use crate::{Metadata, Result};
use std::io::{Read, Seek};

pub trait MetadataFormat<'a>: Sized {
    /// Checks if the stream signature matches this format.
    fn detect_signature<R: Read + Seek>(reader: &mut R) -> Result<bool>;

    /// Parses the stream into a unified `Metadata` object.
    fn parse<R: Read + Seek>(
        buffer: &'a mut Vec<u8>,
        reader: &mut R,
        skip_picture: bool,
    ) -> Result<Metadata<'a>>;
}

// TODO: Add traits for FLAC & ID3 to allow for versioning
