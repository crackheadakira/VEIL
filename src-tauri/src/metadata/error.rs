use std::error;
use std::fmt;
use std::io;
use std::string;
use tokio::task::JoinError;

pub type Result<T> = std::result::Result<T, MetadataError>;

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    StringDecoding(std::string::FromUtf8Error),
    InvalidInput,
}

pub struct MetadataError {
    pub kind: ErrorKind,
}

impl error::Error for MetadataError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(err) => Some(err),
            ErrorKind::StringDecoding(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for MetadataError {
    fn from(err: io::Error) -> MetadataError {
        MetadataError {
            kind: ErrorKind::Io(err),
        }
    }
}

impl From<string::FromUtf8Error> for MetadataError {
    fn from(err: string::FromUtf8Error) -> MetadataError {
        MetadataError {
            kind: ErrorKind::StringDecoding(err),
        }
    }
}

impl fmt::Display for MetadataError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Io(err) => write!(out, "IO error: {}", err),
            ErrorKind::StringDecoding(err) => write!(out, "String decoding error: {}", err),
            ErrorKind::InvalidInput => write!(out, "Invalid input provided"),
        }
    }
}

impl fmt::Debug for MetadataError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "{}", self)
    }
}

impl From<&str> for MetadataError {
    fn from(description: &str) -> Self {
        MetadataError {
            kind: ErrorKind::InvalidInput,
        }
    }
}

impl From<JoinError> for MetadataError {
    fn from(err: JoinError) -> MetadataError {
        MetadataError {
            kind: ErrorKind::InvalidInput, // You can adjust this to a more appropriate error type.
        }
    }
}
