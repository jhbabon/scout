extern crate termion;

use self::termion::{cursor, color, style};
use self::termion::event::Key;
use self::termion::input::TermRead;

use std::io::{self, Write};
use std::fmt;
use choice::Choice;

const MAX_LINES: usize = 21;

pub enum Action {
    DeleteChar,
    MoveUp,
    MoveDown,
    Done,
    Clear,
    Add(char),
    Exit,
}

impl Action {
    // NOTE: We don't use the From trait because we are returning
    // an Option and TryFrom is still experimental.
    pub fn from(key: Key) -> Option<Self> {
        match key {
            Key::Backspace                   => Some(Action::DeleteChar),
            Key::Ctrl('p')  | Key::Up        => Some(Action::MoveUp),
            Key::Ctrl('n')  | Key::Down      => Some(Action::MoveDown),
            Key::Char('\n') | Key::Ctrl('j') => Some(Action::Done),
            Key::Ctrl('u')                   => Some(Action::Clear),
            Key::Char(c)                     => Some(Action::Add(c)),
            Key::Esc                         => Some(Action::Exit),
            _                                => None,
        }
    }
}

struct Line<'a> {
    choice: Choice<'a>,
    selected: bool,
}

impl<'a> fmt::Display for Line<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let highlight_color = color::Fg(color::LightGreen);
        let reset_color = color::Fg(color::Reset);

        // Split the choice string in different areas
        // to highlight the matching part
        let choice = self.choice.to_string();
        let chars = choice.char_indices();
        let mut ended = None;
        let mut line: String = chars.map(|(index, ch)| {
            if index == self.choice.start() && index < self.choice.end() {
                format!("{}{}", highlight_color, ch)
            } else if index == self.choice.end() {
                ended = Some(index);
                format!("{}{}", reset_color, ch)
            } else {
                format!("{}", ch)
            }
        }).collect();

        // Ensure that we stop highlighting things
        if ended.is_none() {
            line = format!("{}{}", line, reset_color);
        }

        if self.selected {
            write!(f, "{}{}{}", style::Invert, line, style::Reset)
        } else {
            write!(f, "{}", line)
        }
    }
}

// Interact with what the user typed in and get the Action
pub fn interact(buffer: Vec<u8>) -> Vec<Option<Action>> {
    buffer.keys()
        .map(|result| result.map(|key| Action::from(key)).unwrap_or(None))
        .collect()
}

// Renders the whole UI
pub fn render<W: Write>(screen: &mut W, query: &str, choices: &Vec<Choice>, selection: usize, total: usize) -> Result<(), io::Error> {
    clear(screen)?;
    render_choices(screen, choices, selection)?;
    render_prompt(screen, query, choices.len(), total)?;

    screen.flush()?;

    Ok(())
}

// Clears the screen
fn clear<W: Write>(screen: &mut W) -> Result<(), io::Error> {
    writeln!(screen, "{}{}", termion::clear::All, cursor::Goto(1, 1))?;

    Ok(())
}

// Renders each choice
fn render_choices<W: Write>(screen: &mut W, choices: &Vec<Choice>, selection: usize) -> Result<(), io::Error> {
    for (index, choice) in choices.iter().take(MAX_LINES).cloned().enumerate() {
        let line = Line { choice, selected: index == selection };
        writeln!(screen, "{}", line)?;
    }

    Ok(())
}

// Renders the prompt line
fn render_prompt<W: Write>(screen: &mut W, query: &str, matches: usize, total: usize) -> Result<(), io::Error> {
    // Go to the beginning again and redraw the prompt.
    // This will put the cursor at the end of it
    let width = format!("{}", total).len();
    let prompt = format!("{:width$} > {}", matches, query, width = width);

    write!(screen, "{}{}", cursor::Goto(1, 1), prompt)?;

    Ok(())
}
