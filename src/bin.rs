extern crate scout;
extern crate docopt;

use std::env;
use std::process;
use std::io::{self, Read, Write};
use std::collections::HashMap;

use scout::{Terminal, Choice};
use scout::ui::{self, Window, Action};

use docopt::Docopt;

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
  $ ls | scout
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
        Ok(_) => {},
        Err(error) => fatal(error),
    };

    let list: Vec<&str> = buffer.split("\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    match magic(list) {
        Ok(result) => println!("{}", result),
        Err(e) => fatal(e),
    }
}

fn magic<'a>(list: Vec<&'a str>) -> Result<String, io::Error> {
    let total = list.len();

    let mut last_actions: Vec<Option<Action>> = vec![];
    let mut terminal = Terminal::new();
    let mut window = Window::new(&terminal, total);
    let mut result = String::new();
    let mut query: Vec<char> = vec![];
    let mut query_string: String;
    let mut history: HashMap<String, Vec<Choice>> = HashMap::new();

    'event: loop {
        window.refine(&last_actions);
        query_string = query.iter().cloned().collect();
        let choices = history.entry(query_string.to_owned())
            .or_insert_with(|| scout::explore(&list, &query));

        ui::render(&mut terminal, &query_string, &choices, &window)?;

        let actions = ui::interact(terminal.input());
        for action in actions.iter().cloned() {
            match action {
                Some(Action::DeleteChar) => {
                    let _ = query.pop();
                },
                Some(Action::Clear) => {
                    query.clear();
                },
                Some(Action::Add(c)) => {
                    query.push(c);
                },
                Some(Action::Done) => {
                    let choice = choices[window.selection()];
                    result = choice.to_string();

                    break 'event
                },
                Some(Action::Exit) => {
                    break 'event
                },
                Some(_) | None => {}
            }
        }

        last_actions = actions;
    };

    Ok(result)
}

fn fatal(error: io::Error) {
    let stderr = io::stderr();
    writeln!(stderr.lock(), "ERROR: {}", error)
        .expect("ERROR while writting to STDERR");

    process::exit(1);
}
