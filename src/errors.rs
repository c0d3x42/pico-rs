use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PicoError {
    IncompatibleComparison,
    NoSuchValue,
    Crash(String),
}

impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
impl Error for PicoError {}
