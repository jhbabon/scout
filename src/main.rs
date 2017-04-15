extern crate termios;

use std::{fs, io};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

fn magic() -> Result<(), io::Error> {
    println!("The magic begins");

    let mut input = String::new();
    try!(io::stdin().read_to_string(&mut input));

    // I need to transform tty into raw mode to get chars byte by byte.
    // Check termios crate
    // see: http://stackoverflow.com/a/37416107
    let mut tty = try!(fs::OpenOptions::new().read(true).write(true).open("/dev/tty"));
    let fd = tty.as_raw_fd();
    let original_tty = Termios::from_fd(fd).unwrap();
    let mut new_tty = original_tty.clone();  // make a mutable copy of termios
                                            // that we will modify
    new_tty.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(fd, TCSANOW, &mut new_tty).unwrap();

    let clear = "\x1B[2J";
    write!(&mut tty, "{}", clear).unwrap();
    writeln!(&mut tty, "Piped input").unwrap();
    writeln!(&mut tty, "{}", input).unwrap();

    write!(&mut tty, "> ").unwrap();
    tty.flush().unwrap();

    'event: loop {
        // Some chars are 2 bytes at a time, so better
        // to read 2 by 2. E.g: Arrow keys
        let mut buffer = [0;1];
        tty.read_exact(&mut buffer).unwrap();
        // let chars: Vec<char> = buffer.iter()
        //                         .map(|&b| b as char)
        //                         .collect();
        match buffer[0] {
            b'\x1B' => break 'event,
            // TODO: Backspace
            // b'\x7F' => break 'event,
            c => write!(&mut tty, "{}", c as char).unwrap(),
        }
    };

    writeln!(&mut tty, "").unwrap();

    tcsetattr(fd, TCSANOW, & original_tty).unwrap();  // reset the stdin to

    Ok(())
}

fn main() {
    magic().unwrap();
}
