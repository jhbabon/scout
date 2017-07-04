use termios::{self, Termios};
use termion;
use termion::screen::AlternateScreen;
use std::io::{self, Read, Write};
use std::fs::File;
use std::os::unix::io::{RawFd, AsRawFd};

use errors::Error;
use terminal_size::terminal_size;

pub trait Measurable {
    fn size(&self) -> (usize, usize);
}

/// Representation of the pseudo terminal used to display scout UI and get user's input.
///
/// This only works on UNIX systems since it opens the pseudo terminal on `/dev/tty`.
pub struct Terminal {
    fd: RawFd,
    tty: Termios,
    alternate: AlternateScreen<File>,
}

impl Terminal {
    /// Build a new Terminal.
    ///
    /// Building the terminal makes the screen to switch to a clean alternate screen ready to
    /// display the UI and get user's input.
    pub fn new() -> Result<Self, Error> {
        let device_tty = termion::get_tty()?;
        let fd = device_tty.as_raw_fd();

        // Modify the tty so it doesn't print back user's input and it takes input without parsing
        // it.
        //
        // We'll call this "raw mode"
        let tty = Termios::from_fd(fd)?;
        let mut raw_tty = tty;
        raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;

        let alternate = AlternateScreen::from(device_tty);
        let terminal = Self { fd, tty, alternate };

        Ok(terminal)
    }

    /// Get the user's input from the terminal.
    ///
    /// This method only reads as much as 4 bytes. The idea is to read char by char, but since
    /// there are sequences like ^ (Cntrl) sequences or arrow keys that use more than one or two
    /// bytes, it's better to cover most of the cases.
    pub fn input(&mut self) -> Vec<u8> {
        // We only want to read as much as 4 bytes
        let mut buffer = [0; 4];

        match self.alternate.read(&mut buffer) {
            Ok(n) => buffer.iter().take(n).cloned().collect(),
            Err(_) => vec![],
        }
    }
}

impl Measurable for Terminal {
    /// Get the terminal size; (width, height)
    fn size(&self) -> (usize, usize) {
        match terminal_size(self.fd) {
            Ok((width, height)) => (width as usize, height as usize),
            Err(_) => (0, 0),
        }
    }
}

impl Drop for Terminal {
    /// Ensure that we restore the /dev/tty to its original config
    /// once the Terminal goes out of scope.
    fn drop(&mut self) {
        termios::tcsetattr(self.fd, termios::TCSANOW, &self.tty)
            .expect("Error restoring the original tty configuration");
    }
}

impl Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.alternate.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.alternate.flush()
    }
}
