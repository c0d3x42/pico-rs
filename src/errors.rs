//use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PicoError {
    #[error("Parse failure in [{filename:?}]")]
    ParseFailure { filename: String },

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

/*impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
*/
//impl Error for PicoError {}
