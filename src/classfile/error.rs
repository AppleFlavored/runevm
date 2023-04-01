use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidIndex(u16),
    InvalidMagic(u32),
    UnhandledConstant(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::IoError(..) => write!(f, "failed to read data from file"),
            Error::InvalidIndex(index) => {
                write!(f, "could not find attribute name at index: {index}")
            }
            Error::InvalidMagic(magic) => write!(f, "file has invalid magic: ({magic})"),
            Error::UnhandledConstant(tag) => write!(f, "reached unhandled constant tag: {tag}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}
