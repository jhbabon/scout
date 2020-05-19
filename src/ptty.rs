//! Pseudo Terminal manipulation

use crate::common::Result;
use async_std::fs;
use async_std::os::unix::io::RawFd;
use log;
use std::convert::TryFrom;
use termios::{self, Termios};

#[derive(Debug)]
pub struct PTTY {
    fd: RawFd,
    termios: Termios,
}

impl PTTY {
    /// In termios terms, noncanonical mode means:
    ///
    /// > In noncanonical mode input is available immediately
    /// > (without the user having to type a line-delimiter character),
    /// > no input processing is performed, and line editing is disabled
    ///
    /// What we want with this method is to have total control on how to
    /// process input and how to print to the pseudo terminal.
    ///
    /// More info in [termios' webpage inside "Canonical and noncanonical mode" section][termios]
    ///
    /// [termios]: https://linux.die.net/man/3/termios
    pub fn noncanonical_mode(&self) -> Result<()> {
        let mut raw_tty = self.termios.clone();
        raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO | termios::ECHONL | termios::IEXTEN);

        termios::tcsetattr(self.fd, termios::TCSANOW, &raw_tty)?;

        Ok(())
    }
}

impl TryFrom<RawFd> for PTTY {
    type Error = std::io::Error;

    fn try_from(fd: RawFd) -> std::result::Result<Self, Self::Error> {
        let termios = Termios::from_fd(fd)?;
        let tty = PTTY { fd, termios };

        Ok(tty)
    }
}

impl Drop for PTTY {
    fn drop(&mut self) {
        log::trace!("dropping: {:?}", self);

        // Make sure we restore termios settings after the PTTY is dropped
        let _r = termios::tcsetattr(self.fd, termios::TCSANOW, &self.termios);
    }
}

/// Get PTTY file representation
pub async fn get_ptty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}
