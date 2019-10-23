use log::trace;
use std::convert::TryFrom;
use async_std::fs;
use async_std::os::unix::io::RawFd;
use termios::{self, Termios};
use crate::result::Result;

#[derive(Debug,Clone)]
pub struct PTTY {
    fd: RawFd,
    termios: Termios,
}

impl PTTY {
    pub fn noncanonical_mode(& self) -> Result<()> {
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
        trace!("Dropping: {:?}", self);

        let _r = termios::tcsetattr(self.fd, termios::TCSANOW, &self.termios);
    }
}

pub async fn get_ptty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}
