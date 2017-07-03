#![warn(missing_docs)]

extern crate scout;
extern crate docopt;

use std::env;
use std::process;
use std::io::{self, Read, Write};

use docopt::Docopt;
use scout::errors::Error;

const USAGE: &'static str = "
Scout: Small fuzzy finder

This program expects a list of items in the standard input,
so it is better to use it with pipes.

Usage:
  scout [options]

Options:
  -h --help     Show this screen.
  -v --version  Show version.

Supported keys:
   * ^U to delete the entire line
   * ^N or Arrow key down to select the next match
   * ^P or Arrow key up to select the previous match
   * ESC to quit without selecting a match

Example:
  $ find * -type f | scout
";

pub fn main() {
    Docopt::new(USAGE)
        .and_then(|doc| {
            doc.argv(env::args())
                .version(Some(scout::version()))
                .parse()
        })
        .unwrap_or_else(|e| e.exit());;

    // Collect initial input
    let mut buffer = String::new();
    let stdin = io::stdin();
    match stdin.lock().read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(error) => fatal(&error.into()),
    };

    let list: Vec<&str> = buffer
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    match scout::start(list) {
        Ok(result) => println!("{}", result),
        Err(error) => fatal(&error),
    }
}

fn fatal(error: &Error) {
    let stderr = io::stderr();
    writeln!(stderr.lock(), "ERROR: {}", error).expect("ERROR while writting to STDERR");

    process::exit(1);
}
