// NOTE: This is a copy and adaptation of the original mod "size" from
// the crate "termion".
//
// The original "termion::terminal_size()" function only checks the size
// against the STDOUT, but to interact with the user we use a custom
// tty (/dev/tty to be precise), so we need to use a different file
// descriptor. I don't know if this "feature" will be available in
// "termion" any time soon, so fo the moment I adapt the original code
// to my needs.
use libc::{c_int, c_ushort};
use std::io;

#[cfg(target_os = "linux")]
pub const TIOCGWINSZ: usize = 0x0000_5413;

#[cfg(not(target_os = "linux"))]
pub const TIOCGWINSZ: usize = 0x40087468;

#[repr(C)]
struct TermSize {
    row: c_ushort,
    col: c_ushort,
    _x: c_ushort,
    _y: c_ushort,
}

#[cfg(target_env = "musl")]
fn tiocgwinsz() -> i32 {
    TIOCGWINSZ as i32
}

#[cfg(all(not(target_env = "musl"), target_pointer_width = "64"))]
fn tiocgwinsz() -> u64 {
    TIOCGWINSZ as u64
}

#[cfg(all(not(target_env = "musl"), target_pointer_width = "32"))]
fn tiocgwinsz() -> u32 {
    TIOCGWINSZ as u32
}

// Get the size of the terminal.
pub fn terminal_size(fileno: c_int) -> io::Result<(u16, u16)> {
    use libc::ioctl;
    use std::mem;

    unsafe {
        let mut size: TermSize = mem::zeroed();

        if ioctl(fileno, tiocgwinsz(), &mut size as *mut _) == 0 {
            Ok((size.col as u16, size.row as u16))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to get the terminal size.",
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_size() {
        use libc::STDOUT_FILENO;

        assert!(terminal_size(STDOUT_FILENO).is_ok());
    }
}
