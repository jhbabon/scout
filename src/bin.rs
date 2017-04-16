// TODO: Clean code, please
// TODO: Check types used. Try to minimize the memory used
// TODO: Add more tests.
// TODO: Handle empty imput
// TODO: Better UI. Colors? Num of matches?
// TODO: Try to do the fuzzy search async?
extern crate scout;
extern crate docopt;
extern crate termios;
extern crate termion;

use std::env;

use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use termion::event::Key;
use termion::input::TermRead;
use termion::screen::*;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;

const USAGE: &'static str = "
Scout: Small fuzzy finder

This program expects a list of items in the
standard input, so it is better to use it
with pipes.

Usage:
  scout [options]

Options:
  -h --help      Show this screen.
  -v --version   Show version.

Example:
  $ ls | scout
";

fn magic() -> Result<String, io::Error> {
    // Collect initial input
    let mut buffer = String::new();
    try!(io::stdin().read_to_string(&mut buffer));
    let input: Vec<&str> = buffer.split("\n").collect();

    // I need to transform tty into raw mode to get chars byte by byte.
    // Check termios crate
    // see: http://stackoverflow.com/a/37416107
    let tty = termion::get_tty().unwrap();
    let fd = tty.as_raw_fd();
    let original_tty = Termios::from_fd(fd).unwrap();
    let mut new_tty = original_tty.clone();  // make a mutable copy of termios
                                            // that we will modify
    new_tty.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(fd, TCSANOW, &mut new_tty).unwrap();

    // Then we can use this tty to create our screen with the help
    // of termion crate
    let mut screen = AlternateScreen::from(tty);
    let mut result = String::new();
    let mut query: Vec<char> = vec![];
    let mut buffer: Vec<u8> = Vec::with_capacity(2);

    'event: loop {
        let s: String = query.iter().cloned().collect();
        let query_chars: Vec<char> = query.iter().cloned().collect();
        let suggestions = scout::explore(&input, &query_chars);

        write!(
            &mut screen,
            "{}{}> {}\n",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            s,
        ).unwrap();

        for item in suggestions.iter().take(21).cloned() {
            writeln!(&mut screen, "{}", item).unwrap();
        }

        write!(
            &mut screen,
            "{}",
            termion::cursor::Goto((s.len() + 3) as u16, 1),
        ).unwrap();

        screen.flush().unwrap();

        // Some chars are 2 bytes at a time, so better
        // to read 2 by 2. E.g: Arrow keys
        let mut int_buffer = [0;2];
        buffer.clear();
        // Ensure that we read 2 bytes in an intermediate buffer
        //
        // If the amount of real bytes read is less than 2,
        // put only that byte in the buffer so we don't carry
        // junk.
        //
        // If the amount is 2, put both in the buffer
        match screen.read(&mut int_buffer) {
            Ok(1) => {
                buffer.push(int_buffer[0])
            },
            Ok(2) => {
                buffer = int_buffer.iter().map(|&x| x).collect()
            }
            Ok(_) => {},
            Err(_) => {}
        };

        // Now this buffer has the exact amount of bytes from
        // the input so we can use the #keys() function from termion
        // to read and transform those bytes into proper keys
        for c in buffer.keys() {
            match c.unwrap() {
                Key::Esc => break 'event,
                Key::Backspace => {
                    let _ = query.pop();
                },
                Key::Char('\n') => {
                    // result = query.iter().cloned().collect();
                    if let Some(&choice) = suggestions.first() {
                        result = String::from(choice);
                    }
                    break 'event
                },
                Key::Char(c) => {
                    query.push(c as char);
                },
                _ => {
                    query.push('?');
                },
            }
        }
    };

    tcsetattr(fd, TCSANOW, & original_tty).unwrap();  // reset the stdin to

    Ok(result)
}

pub fn main() {
    docopt::Docopt::new(USAGE)
        .and_then(|doc| {
            doc.argv(env::args())
                .version(Some(scout::version()))
                .parse()
        })
        .unwrap_or_else(|e| e.exit());;

    match magic() {
        Ok(result) => println!("{}", result),
        Err(e) => println!("{:?}", e),
    }
}
