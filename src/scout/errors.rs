use std::{fmt, error, io};

use regex;

pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Internal error")
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Internal error")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Something happened!"
    }
}

impl From<regex::Error> for Error {
    fn from(_error: regex::Error) -> Self {
        Error
    }
}

impl From<io::Error> for Error {
    fn from(_error: io::Error) -> Self {
        Error
    }
}
