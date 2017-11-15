#![warn(missing_docs)]

//! The command line entry point.
//!
//! It reads the STDIN and writes to STDOUT the choice selected. In
//! case of error, it will print it to STDERR.

extern crate docopt;
extern crate scout;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::process;
use std::io::{self, Read, Write};

use docopt::Docopt;
use scout::errors::Error;

const USAGE: &str = "
Scout: Small fuzzy finder

This program expects a list of items in the standard input,
so it is better to use it with pipes.

Usage:
  scout [--search=<query>]
  scout -h | --help
  scout -v | --version

Options:
  -s --search=<query>  Start the search with the given query
  -h --help            Show this screen.
  -v --version         Show version.

Supported keys:
   * ^U to delete the entire line
   * ^N or Arrow key down to select the next match
   * ^P or Arrow key up to select the previous match
   * ESC to quit without selecting a match

Example:
  $ find * -type f | scout

  # Pass an initial query to start filtering right away
  $ find * -type f | scout --search=foo
";

#[derive(Deserialize)]
struct Args {
    flag_search: Option<String>,
}

/// Start the CLI.
pub fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|doc| {
            doc.argv(env::args())
                .version(Some(scout::version()))
                .deserialize()
        })
        .unwrap_or_else(|e| e.exit());

    let query = match args.flag_search {
        Some(q) => q.chars().collect::<Vec<char>>(),
        None => vec![],
    };

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

    match scout::start(list, query) {
        Ok(result) => println!("{}", result),
        Err(error) => fatal(&error),
    }
}

/// Print the error to STDERR and stop the program in a non zero exit status.
fn fatal(error: &Error) {
    let stderr = io::stderr();
    writeln!(stderr.lock(), "ERROR: {}", error).expect("ERROR while writting to STDERR");

    process::exit(1);
}
