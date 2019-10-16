use async_std::fs;
use async_std::os::unix::io::RawFd;

use termios::{self, Termios};

use crate::result::Result;

pub async fn get_tty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}

pub fn into_raw_input(fd: RawFd) -> Result<()> {
    let mut raw_tty = Termios::from_fd(fd)?;
    raw_tty.c_lflag &= !(
        termios::ICANON |
        termios::ECHO |
        termios::ECHONL |
        termios::IEXTEN
    );
    termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;

    Ok(())
}

pub fn into_raw_output(fd: RawFd) -> Result<()> {
    let mut raw_tty = Termios::from_fd(fd)?;
    raw_tty.c_lflag &= !(
        termios::ICANON |
        termios::ECHO
    );
    termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;

    Ok(())
}
