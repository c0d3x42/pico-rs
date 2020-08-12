//use std::fmt;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PicoError {
    #[error("Parse failure in [{filename:?}]")]
    ParseFailure { filename: String },

    #[error("Unusable included file [{filename:?}]")]
    UnusableFile { filename: String },

    #[error(transparent)]
    SerDe(#[from] serde_json::Error),

    #[error("Cant compare apples to oranges")]
    IncompatibleComparison,

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

    #[error("Failed to parse [{filename:?}]")]
    ParseError {
        source: serde_json::Error,
        filename: String,
    },

    #[error("unknown data store error")]
    Unknown(#[from] anyhow::Error),
}

/*impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
*/
//impl Error for PicoError {}
