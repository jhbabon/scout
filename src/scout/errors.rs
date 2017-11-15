//! Behold! All errors live here

use std::{error, fmt, io};
use regex;

/// Main scout error struct
pub struct Error {
    display: String,
    debug: String,
    description: String,
}

impl Error {
    /// Create a new error from any other by importing its information.
    pub fn import<E: fmt::Display + fmt::Debug + error::Error>(error: E) -> Self {
        let display = format!("{}", error);
        let debug = format!("{:?}", error);
        let description = error.description().to_owned();

        Error {
            display,
            debug,
            description,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.display)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.debug)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::import(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::import(error)
    }
}
