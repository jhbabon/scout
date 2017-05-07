// TODO: Clean code, please
// TODO: Check types used. Try to minimize the memory used
// TODO: Add more tests.
// TODO: Better UI. Colors? Num of matches?
// TODO: Try to do the fuzzy search async?
extern crate scout;
extern crate docopt;
extern crate termios;
extern crate termion;

use std::env;

use termion::event::Key;
use termion::input::TermRead;
use termion::{color, style};
use std::io::{self, Read, Write};

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
    let width = format!("{}", input.len()).len();
    let mut selection = 0; // current selected item

    let mut terminal = scout::Terminal::new();
    let mut result = String::new();
    let mut query: Vec<char> = vec![];

    'event: loop {
        let s: String = query.iter().cloned().collect();
        let query_chars: Vec<char> = query.iter().cloned().collect();
        let choices = scout::explore(&input, &query_chars);

        // Clear the terminal and put the cursor at the beginning
        writeln!(
            &mut terminal,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        ).unwrap();

        // Print all the choices
        for (i, choice) in choices.iter().take(21).cloned().enumerate() {
            // Split the string in different areas
            // to highlight the matching part
            let string = choice.to_string();
            let chars = string.char_indices();
            let mut ended = None;
            let mut line: String = chars.map(|(index, ch)| {
                if index == choice.start() && index < choice.end() {
                    format!("{}{}", color::Fg(color::LightGreen), ch)
                } else if index == choice.end() {
                    ended = Some(index);
                    format!("{}{}", color::Fg(color::Reset), ch)
                } else {
                    format!("{}", ch)
                }
            }).collect();

            // Ensure that we stop highlihting things
            if ended.is_none() {
                line = format!("{}{}", line, color::Fg(color::Reset));
            }

            if i == selection {
                writeln!(&mut terminal, "{}{}{}", style::Invert, line, style::Reset).unwrap();
            } else {
                writeln!(&mut terminal, "{}", line).unwrap();
            }
        }

        // Go to the beginning again and redraw the prompt.
        // This will put the cursor at the end of it
        let prompt = format!("{:width$} > {}", choices.len(), s, width = width);
        write!(&mut terminal, "{}{}", termion::cursor::Goto(1, 1), prompt).unwrap();

        terminal.flush().unwrap();

        for c in terminal.input().keys() {
            debug!("In for loop");
            match c.unwrap() {
                Key::Backspace => {
                    let _ = query.pop();
                },
                Key::Ctrl('p') | Key::Up => {
                    selection = if selection == 0 {
                        // TODO: This should be only over the visible
                        // window
                        choices.len() - 1
                    } else {
                        selection - 1
                    };
                },
                Key::Ctrl('n') | Key::Down => {
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
                Key::Char('\n') | Key::Ctrl('j') | Key::Ctrl('m') => {
                    let choice = choices[selection];
                    result = choice.to_string();

                    break 'event
                },
                Key::Ctrl('u') => {
                    query.clear();
                },
                Key::Char(c) => {
                    query.push(c);
                },
                Key::Esc => break 'event,
                _ => {},
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
