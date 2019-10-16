use log::debug;

use std::convert::TryFrom;

use async_std::fs;
use async_std::os::unix::io::RawFd;

use termios::{self, Termios};

use crate::result::Result;

#[derive(Debug,Clone)]
pub struct TTY {
    fd: RawFd,
    termios: Termios,
}

impl TTY {
    pub fn into_raw(& self) -> Result<()> {
        let mut raw_tty = self.termios.clone();
        raw_tty.c_lflag &= !(
            termios::ICANON |
            termios::ECHO |
            termios::ECHONL |
            termios::IEXTEN
        );
        termios::tcsetattr(self.fd, termios::TCSANOW, &raw_tty)?;

        Ok(())
    }
}

impl TryFrom<RawFd> for TTY {
    type Error = std::io::Error;

    fn try_from(fd: RawFd) -> std::result::Result<Self, Self::Error> {
        let termios = Termios::from_fd(fd)?;
        let tty = TTY { fd, termios };

        Ok(tty)
    }
}

impl Drop for TTY {
    fn drop(&mut self) {
        debug!("Dropping TTY: {:?}", self);

        let _r = termios::tcsetattr(self.fd, termios::TCSANOW, &self.termios);
    }
}

pub async fn get_tty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}
