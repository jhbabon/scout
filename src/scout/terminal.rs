use termios::{self, Termios};
use termion;
use termion::screen::AlternateScreen;
use std::io::{self, Read, Write};
use std::fs::File;
use std::os::unix::io::{RawFd, AsRawFd};
use std::default::Default;

use terminal_size::terminal_size;

pub struct Terminal {
    fd: RawFd, // File Descriptor of /dev/tty
    tty: Termios, // Termios /dev/tty representation
    alternate: AlternateScreen<File>, // Termion AlternateScreen
}

impl Terminal {
    // TODO: Use Result
    pub fn new() -> Self {
        let dev_tty = termion::get_tty().unwrap();
        let fd = dev_tty.as_raw_fd();

        // Modify the tty so it doesn't print back
        // user's input and it takes input without
        // parsing it.
        //
        // We'll call this "raw mode"
        let tty = Termios::from_fd(fd).unwrap();
        let mut raw_tty = tty;
        raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios::tcsetattr(fd, termios::TCSANOW, &raw_tty).unwrap();

        let alternate = AlternateScreen::from(dev_tty);

        Terminal { fd, tty, alternate }
    }

    pub fn input(&mut self) -> Vec<u8> {
        let mut internal = [0; 4];
        let mut buffer: Vec<u8> = vec![];

        if let Ok(n) = self.alternate.read(&mut internal) {
            buffer = internal.iter().take(n).cloned().collect()
        };

        buffer
    }

    pub fn size(&self) -> (usize, usize) {
        match terminal_size(self.fd) {
            Ok((width, height)) => (width as usize, height as usize),
            Err(_) => (0, 0),
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Restore the /dev/tty to its original config
        termios::tcsetattr(self.fd, termios::TCSANOW, &self.tty).unwrap();
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

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}
