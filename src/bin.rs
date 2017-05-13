// TODO: Clean code, please
// TODO: Check types used. Try to minimize the memory used
// TODO: Add more tests.
// TODO: Better UI. Colors? Num of matches?
// TODO: Try to do the fuzzy search async?
extern crate scout;
extern crate docopt;

use std::env;
use std::io::{self, Read};

use scout::ui::Action;

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
    let input: Vec<&str> = buffer.split("\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let total = input.len();
    let mut selection = 0; // current selected item

    let mut terminal = scout::Terminal::new();
    let mut result = String::new();
    let mut query: Vec<char> = vec![];

    'event: loop {
        let s: String = query.iter().cloned().collect();
        let query_chars: Vec<char> = query.iter().cloned().collect();
        let choices = scout::explore(&input, &query_chars);

        scout::ui::render(&mut terminal, &s, &choices, selection, total)?;

        let inputs = scout::ui::interact(terminal.input());
        for input in inputs {
            match input {
                Some(Action::DeleteChar) => {
                    let _ = query.pop();
                },
                Some(Action::MoveUp) => {
                    selection = if selection == 0 {
                        // TODO: This should be only over the visible
                        // window
                        choices.len() - 1
                    } else {
                        selection - 1
                    };
                },
                Some(Action::MoveDown) => {
                    // TODO: This should be only over the visible
                    // window
                    // TODO: The loop shouldn't be trigger again,
                    // we should render the terminal without
                    // doing a full search. In the next event loop
                    // iteration the selection should be reset to
                    // 0 again
                    selection = if selection == (choices.len() - 1) {
                        0
                    } else {
                        selection + 1
                    };
                },
                Some(Action::Clear) => {
                    query.clear();
                },
                Some(Action::Add(c)) => {
                    query.push(c);
                },
                Some(Action::Done) => {
                    let choice = choices[selection];
                    result = choice.to_string();

                    break 'event
                },
                Some(Action::Exit) => break 'event,
                None => {}
            }
        }

    };

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
        Err(e) => panic!(e),
    }
}
