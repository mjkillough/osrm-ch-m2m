use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::path::PathBuf;
use std::result;

#[derive(Debug)]
pub enum Error {
    Message(&'static str),
    IoError(std::io::Error),
    Serde(serde_json::Error),
    MissingFile { tar: PathBuf, path: PathBuf },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(inner) => inner.fmt(f),
            Error::IoError(inner) => inner.fmt(f),
            Error::Serde(inner) => inner.fmt(f),
            Error::MissingFile { tar, path } => write!(f, "Not found in {:?}: {:?}", tar, path),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Message(_) => None,
            Error::IoError(inner) => Some(inner),
            Error::Serde(inner) => Some(inner),
            Error::MissingFile { .. } => None,
        }
    }
}

impl From<&'static str> for Error {
    fn from(string: &'static str) -> Error {
        Error::Message(string)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Serde(error)
    }
}

pub type Result<T> = result::Result<T, Error>;
