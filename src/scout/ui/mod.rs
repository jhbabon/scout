//! Collection of functions and structs to build the User Interface

mod action;
mod window;
mod line;

use termion::{self, cursor};
use termion::input::TermRead;

use std::io::Write;
use choice::Choice;
use errors::Error;

use self::line::Line;
pub use self::action::Action;
pub use self::window::Window;

/// Interact with what the raw user's input and get the corresponding Actions
///
/// # Example
///
/// ```
/// use scout::ui;
///
/// let buffer = b"a\n";
/// let expected = vec![ui::Action::Add('a'), ui::Action::Done];
///
/// assert_eq!(expected, ui::interact(buffer));
/// ```
pub fn interact(buffer: &[u8]) -> Vec<Action> {
    buffer
        .keys()
        .map(|result| result.map(Action::from).unwrap_or(None))
        .filter_map(|action| action)
        .collect()
}

/// Render the whole UI
///
/// Given the following elements:
///
/// * query: `['a', 'b', 'c']`
/// * choices: `["a/b/c.rs", "a/a/b/c.rs"]`
///
/// And a screen with enough space, the UI would look like this (`|` represents the cursor):
///
/// ```text
///  2 > abc|
/// a/b/c.rs
/// a/a/b/c.rs
/// ```
///
/// It will have also colors for the matching areas in the choices and the current selection
/// highlighted as well.
pub fn render<W: Write>(
    screen: &mut W,
    query: &str,
    choices: &[Choice],
    window: &Window,
) -> Result<(), Error> {
    clear(screen)?;
    render_choices(screen, choices, window)?;
    render_prompt(screen, query, choices.len(), window)?;

    screen.flush()?;

    Ok(())
}

/// Clears the screen
fn clear<W: Write>(screen: &mut W) -> Result<(), Error> {
    writeln!(screen, "{}{}", termion::clear::All, cursor::Goto(1, 1))?;

    Ok(())
}

/// Renders each choice
fn render_choices<W: Write>(
    screen: &mut W,
    choices: &[Choice],
    window: &Window,
) -> Result<(), Error> {
    let visible_choices = choices
        .iter()
        .cloned()
        .enumerate()
        .skip(window.offset())
        .take(window.lines_len());

    for (index, choice) in visible_choices {
        let line = Line::new(choice, index, window);
        writeln!(screen, "{}", line)?;
    }

    Ok(())
}

/// Renders the prompt line
///
/// The prompt line is at the beginning, the first line. To draw it this method just moves the
/// cursor to the first line and writes the prompt (num of choices and current query). Because is
/// the last thing to write, the cursor will stay at the end of the prompt line.
fn render_prompt<W: Write>(
    screen: &mut W,
    query: &str,
    matches: usize,
    window: &Window,
) -> Result<(), Error> {
    let width = window.prompt_width();
    let prompt = format!("{:width$} > {}", matches, query, width = width);

    write!(screen, "{}{}", cursor::Goto(1, 1), prompt)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use termion::{color, style};
    use super::*;
    use terminal::Measurable;

    struct DummyTerminal {
        width: usize,
        height: usize,
    }

    impl Measurable for DummyTerminal {
        fn size(&self) -> (usize, usize) {
            (self.width, self.height)
        }
    }

    #[test]
    fn it_renders_the_ui() {
        // First we print the list of choices, then the
        // prompt using control sequences to change the
        // position of the cursor.
        //
        // This would look like this (with colors):
        //
        //   2 > abc
        //   a/b/c.rs
        //
        // Only one choice is displayed because the window
        // is created without enough height
        let expected = format!(
            "{}{}\n{}{}a/b/c{}.rs{}\n{} 2 > abc",
            termion::clear::All,
            cursor::Goto(1, 1),
            style::Invert,
            color::Fg(color::LightGreen),
            color::Fg(color::Reset),
            style::Reset,
            cursor::Goto(1, 1),
        );

        let mut screen: Vec<u8> = vec![];
        let query = "abc";
        let choices = [
            Choice::new("a/b/c.rs".to_string(), 0, 5),
            Choice::new("foo/a/b/c.rs".to_string(), 4, 9),
        ];

        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 3,
        };
        let window = Window::new(&terminal, input_len);

        assert!(render(&mut screen, query, &choices, &window).is_ok());

        let actual = String::from_utf8(screen).unwrap();
        assert_eq!(expected, actual);
    }
}
