//use std::fmt;
use thiserror::Error;

use crate::values::PicoValue;

#[derive(Error, Debug)]
pub enum PicoError {
    #[error("Parse failure in [{filename:?}]")]
    ParseFailure { filename: String },

    #[error("Unusable included file [{filename:?}]")]
    UnusableFile { filename: String },

    #[error(transparent)]
    SerDe(#[from] serde_json::Error),

    #[error("Cant compare apples to oranges `{0}`, `{1}` ")]
    IncompatibleComparison(PicoValue, PicoValue),

    #[error("Value `{0}` did not exist")]
    NoSuchValue(String),

    #[error("Read error")]
    ReadError { source: std::io::Error },

    #[error("XXXXX")]
    IOError(#[from] std::io::Error),

    #[error("kkkk")]
    AnyError(#[from] anyhow::Error),

    #[error("Fatal failure")]
    Crash(String),
}

#[derive(Debug, Error)]
pub enum RuleFileError {
    #[error("MyRead Error [{filename:?}]")]
    ReadError {
        source: std::io::Error,
        filename: String,
    },

    #[error("CCCC")]
    IOError(#[from] std::io::Error),

    #[error("Failed to parse [{filename:?}]")]
    ParseError {
        source: serde_json::Error,
        filename: String,
    },

    #[error("Recursive include [{filename:?}]")]
    RecursiveInclude { filename: String },

    #[error("unknown data store error")]
    Unknown(#[from] anyhow::Error),

    #[error("Unsuported [{url:?}]")]
    Unsuported { url: String },
}

/*impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
*/
//impl Error for PicoError {}
