#![warn(missing_docs)]

/*!
Scout is a small fuzzy finder for the command line.

It allows you to select an item from a list of choices with as few characters as possible.

Scout takes the list of choices from the standard input. Because of
this, it opens a new pseudo terminal to allow the user to interact with the program. Once the
user selects a choice (press Enter), scout closes the pseudo terminal, returns to the original
terminal and prints out the choice to the standard output.

The scout library is divided between the elements to control the [`UI`](ui/index.html), the
[`Terminal`](struct.Terminal.html) change and [the engine](struct.Scout.html) to perform the fuzzy
filtering.
*/

extern crate libc;
extern crate termios;
extern crate termion;
extern crate regex;
extern crate num_cpus;
extern crate futures;
extern crate futures_cpupool;

use std::collections::HashMap;

mod score;
mod choice;
mod pattern;
mod terminal_size;
mod terminal;
mod scout;
mod refine;

pub mod ui;
pub mod errors;
pub use choice::Choice;
pub use terminal::Terminal;
pub use scout::Scout;

pub fn start(list: Vec<&str>) -> Result<String, errors::Error> {
    let total = list.len();

    let mut last_actions: Vec<ui::Action> = vec![];
    let mut terminal = Terminal::new()?;
    let mut window = ui::Window::new(&terminal, total);
    let mut result = String::new();
    let mut query: Vec<char> = vec![];
    let mut query_string: String;
    let mut history: HashMap<String, Vec<Choice>> = HashMap::new();
    let scout = Scout::new(list);

    'event: loop {
        query_string = query.iter().cloned().collect();
        let choices = history
            .entry(query_string.to_owned())
            .or_insert_with(|| scout.explore(&query));

        window.outline(&last_actions, choices.len());
        ui::render(&mut terminal, &query_string, choices, &window)?;

        let actions = ui::interact(&terminal.input());
        for action in actions.iter().cloned() {
            match action {
                ui::Action::DeleteChar => {
                    let _ = query.pop();
                }
                ui::Action::Clear => {
                    query.clear();
                }
                ui::Action::Add(c) => {
                    query.push(c);
                }
                ui::Action::Done => {
                    let choice = &choices[window.selection()];
                    result = choice.to_string();

                    break 'event;
                }
                ui::Action::Exit => break 'event,
                _ => {}
            }
        }

        last_actions = actions;
    }

    Ok(result)
}

/// Get the version of the lib.
pub fn version() -> String {
    let (maj, min, pat) = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH"),
    );

    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_string(),
    }
}
