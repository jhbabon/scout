extern crate scout;
extern crate docopt;

use std::env;
use std::io::{self, Read};
use std::collections::HashMap;

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

    let mut window: scout::ui::Window = total.into();
    let mut last_actions: Vec<Option<Action>> = vec![];
    let mut terminal = scout::Terminal::new();
    let mut result = String::new();
    let mut query: Vec<char> = vec![];
    let mut query_string: String;
    let mut history: HashMap<String, Vec<scout::Choice>> = HashMap::new();

    'event: loop {
        window.refine(&last_actions);
        query_string = query.iter().cloned().collect();
        let choices = history.entry(query_string.to_owned())
            .or_insert_with(|| scout::explore(&input, &query));

        scout::ui::render(&mut terminal, &query_string, &choices, &window)?;

        let actions = scout::ui::interact(terminal.input());
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
                Some(Action::Exit) => break 'event,
                Some(_) | None => {}
            }
        }

        last_actions = actions;
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
