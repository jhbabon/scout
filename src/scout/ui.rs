extern crate termion;

use self::termion::{cursor, color, style};
use self::termion::event::Key;
use self::termion::input::TermRead;

use std::io::{self, Write};
use std::fmt;
use choice::Choice;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Default, Debug)]
pub struct Window {
    prompt_width: usize,
    width: usize,
    height: usize,
    selection: usize,
}

impl Window {
    pub fn refine(&mut self, actions: &Vec<Option<Action>>) {
        let max_index = self.choices_len() - 1;
        let mut new_selection = self.selection();

        for action in actions {
            new_selection = match action {
                &Some(Action::MoveUp) => {
                    if new_selection == 0 {
                        max_index
                    } else {
                        new_selection - 1
                    }
                },
                &Some(Action::MoveDown) => {
                    if new_selection == max_index {
                        0
                    } else {
                        new_selection + 1
                    }
                },
                &Some(_) | &None => 0
            }
        }

        self.selection = new_selection;
    }

    pub fn prompt_width(&self) -> usize {
        self.prompt_width
    }

    pub fn selection(&self) -> usize {
        self.selection
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn choices_len(&self) -> usize {
        self.height() - 2
    }
}

impl From<usize> for Window {
    fn from(input_length: usize) -> Self {
        let prompt_width = format!("{}", input_length).len();
        let (width, height) = match termion::terminal_size() {
            Ok((w, h)) => (w as usize, h as usize),
            Err(_) => (0, 0),
        };
        let selection = 0;

        Window { prompt_width, width, height, selection }
    }
}

struct Line<'a> {
    choice: Choice<'a>,
    width: usize,
    selected: bool,
}

impl<'a> Line<'a> {
    pub fn new(choice: Choice<'a>, position: usize, window: &Window) -> Self {
        let width = window.width();
        let selected = position == window.selection();

        Self { choice, width, selected }
    }
}

impl<'a> fmt::Display for Line<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let highlight_color = color::Fg(color::LightGreen);
        let reset_color = color::Fg(color::Reset);

        // Split the choice string in different areas
        // to highlight the matching part
        let choice = self.choice.to_string();
        let chars = choice.char_indices().take(self.width);
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
pub fn render<W: Write>(screen: &mut W, query: &str, choices: &Vec<Choice>, window: &Window) -> Result<(), io::Error> {
    clear(screen)?;
    render_choices(screen, choices, window)?;
    render_prompt(screen, query, choices.len(), window)?;

    screen.flush()?;

    Ok(())
}

// Clears the screen
fn clear<W: Write>(screen: &mut W) -> Result<(), io::Error> {
    writeln!(screen, "{}{}", termion::clear::All, cursor::Goto(1, 1))?;

    Ok(())
}

// Renders each choice
fn render_choices<W: Write>(screen: &mut W, choices: &Vec<Choice>, window: &Window) -> Result<(), io::Error> {
    for (index, choice) in choices.iter().take(window.choices_len()).cloned().enumerate() {
        let line = Line::new(choice, index, window);
        writeln!(screen, "{}", line)?;
    }

    Ok(())
}

// Renders the prompt line
fn render_prompt<W: Write>(screen: &mut W, query: &str, matches: usize, window: &Window) -> Result<(), io::Error> {
    // Go to the beginning again and redraw the prompt.
    // This will put the cursor at the end of it
    let width = window.prompt_width();
    let prompt = format!("{:width$} > {}", matches, query, width = width);

    write!(screen, "{}{}", cursor::Goto(1, 1), prompt)?;

    Ok(())
}
