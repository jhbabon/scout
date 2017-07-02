use termion::{self, cursor, color, style};
use termion::event::Key;
use termion::input::TermRead;

use std::io::{self, Write};
use std::fmt;
use choice::Choice;
use terminal::Measurable;

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
            Key::Backspace => Some(Action::DeleteChar),
            Key::Ctrl('u') => Some(Action::Clear),
            Key::Ctrl('n') | Key::Down => Some(Action::MoveDown),
            Key::Ctrl('p') | Key::Up => Some(Action::MoveUp),
            Key::Char('\n') => Some(Action::Done),
            Key::Ctrl('c') | Key::Esc => Some(Action::Exit),
            Key::Char(c) => Some(Action::Add(c)),
            _ => None,
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
    pub fn new<T: Measurable>(terminal: &T, input_len: usize) -> Self {
        let prompt_width = format!("{}", input_len).len();
        let (width, height) = terminal.size();
        let selection = 0;

        Window {
            prompt_width,
            width,
            height,
            selection,
        }
    }

    pub fn outline(&mut self, actions: &[Option<Action>], choices_len: usize) {
        let max_choices = if choices_len >= self.lines_len() {
            self.lines_len()
        } else {
            choices_len
        };

        let max_index = if max_choices == 0 {
            0
        } else {
            max_choices - 1
        };

        let mut new_selection = self.selection();

        for action in actions {
            new_selection = match *action {
                Some(Action::MoveUp) => {
                    if new_selection == 0 {
                        max_index
                    } else {
                        new_selection - 1
                    }
                }
                Some(Action::MoveDown) => {
                    if new_selection == max_index {
                        0
                    } else {
                        new_selection + 1
                    }
                }
                Some(_) | None => 0,
            }
        }

        self.set_selection(new_selection);
    }

    pub fn prompt_width(&self) -> usize {
        self.prompt_width
    }

    pub fn selection(&self) -> usize {
        self.selection
    }

    fn set_selection(&mut self, new_selection: usize) {
        self.selection = new_selection;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn lines_len(&self) -> usize {
        if self.height() > 1 {
            self.height() - 2
        } else {
            0
        }
    }
}

struct Line {
    choice: Choice,
    width: usize,
    selected: bool,
}

impl Line {
    pub fn new(choice: Choice, position: usize, window: &Window) -> Self {
        let width = window.width();
        let selected = position == window.selection();

        Self {
            choice,
            width,
            selected,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let highlight_color = color::Fg(color::LightGreen);
        let reset_color = color::Fg(color::Reset);

        // Split the choice string in different areas
        // to highlight the matching part
        let choice = self.choice.to_string();
        let chars = choice.char_indices().take(self.width);
        let mut ended = None;
        let mut line: String = chars
            .map(|(index, ch)| {
                if index == self.choice.start() && index < self.choice.end() {
                    format!("{}{}", highlight_color, ch)
                } else if index == self.choice.end() {
                    ended = Some(index);
                    format!("{}{}", reset_color, ch)
                } else {
                    format!("{}", ch)
                }
            })
            .collect();

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

// Interact with what the user typed in and get the Actions
pub fn interact(buffer: &[u8]) -> Vec<Option<Action>> {
    buffer
        .keys()
        .map(|result| result.map(Action::from).unwrap_or(None))
        .collect()
}

// Renders the whole UI
pub fn render<W: Write>(
    screen: &mut W,
    query: &str,
    choices: &[Choice],
    window: &Window,
) -> Result<(), io::Error> {
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
fn render_choices<W: Write>(
    screen: &mut W,
    choices: &[Choice],
    window: &Window,
) -> Result<(), io::Error> {
    for (index, choice) in choices.iter().take(window.lines_len()).cloned().enumerate() {
        let line = Line::new(choice, index, window);
        writeln!(screen, "{}", line)?;
    }

    Ok(())
}

// Renders the prompt line
fn render_prompt<W: Write>(
    screen: &mut W,
    query: &str,
    matches: usize,
    window: &Window,
) -> Result<(), io::Error> {
    // Go to the beginning again and redraw the prompt.
    // This will put the cursor at the end of it
    let width = window.prompt_width();
    let prompt = format!("{:width$} > {}", matches, query, width = width);

    write!(screen, "{}{}", cursor::Goto(1, 1), prompt)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
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
    fn window_is_built_with_correct_proportions() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 30,
        };

        let window = Window::new(&terminal, input_len);

        assert_eq!(2, window.prompt_width());
        assert_eq!(100, window.width());
        assert_eq!(30, window.height());
        assert_eq!(0, window.selection());
    }

    #[test]
    fn window_lines_len_is_proportional_to_its_height() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 30,
        };

        let window = Window::new(&terminal, input_len);

        assert_eq!(28, window.lines_len());
    }

    #[test]
    fn window_lines_len_is_zero_on_small_terminals() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 1,
        };

        let window = Window::new(&terminal, input_len);

        assert_eq!(0, window.lines_len());
    }

    #[test]
    fn window_moves_the_selection_down_when_refining() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let choices_len = 10;

        let mut window = Window::new(&terminal, input_len);
        let actions = [Some(Action::MoveDown)];

        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
    }

    #[test]
    fn window_moves_the_selection_back_up_when_refining() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let choices_len = 10;

        let mut window = Window::new(&terminal, input_len);
        let actions = [Some(Action::MoveDown), Some(Action::MoveUp)];

        window.outline(&actions, choices_len);

        assert_eq!(0, window.selection());
    }

    #[test]
    fn window_cycles_through_the_selections_when_refining() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let choices_len = 10;

        let mut window = Window::new(&terminal, input_len);

        let actions = [Some(Action::MoveDown), Some(Action::MoveDown)];
        window.outline(&actions, choices_len);

        assert_eq!(0, window.selection());

        let actions = [Some(Action::MoveUp)];
        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
    }

    #[test]
    fn window_cycles_through_the_selections_when_refining_using_choices_len_when_necessary() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 40,
        };
        let choices_len = 2;

        let mut window = Window::new(&terminal, input_len);

        let actions = [Some(Action::MoveDown), Some(Action::MoveDown)];
        window.outline(&actions, choices_len);

        assert_eq!(0, window.selection());

        let actions = [Some(Action::MoveUp)];
        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
    }

    #[test]
    fn line_highlights_the_choice_matching_section() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let window = Window::new(&terminal, input_len);
        let position = 1;

        let choice = Choice::new("sample_file.rs".to_string(), 2, 8);

        let line = Line::new(choice, position, &window);

        let expected = format!(
            "sa{}mple_f{}ile.rs",
            color::Fg(color::LightGreen),
            color::Fg(color::Reset)
        );
        let mut actual = String::new();

        write!(&mut actual, "{}", line).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn line_highlights_the_choice_matching_section_on_small_width() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 6,
            height: 4,
        };
        let window = Window::new(&terminal, input_len);
        let position = 1;

        let choice = Choice::new("sample_file.rs".to_string(), 2, 8);

        let line = Line::new(choice, position, &window);

        let expected = format!(
            "sa{}mple{}",
            color::Fg(color::LightGreen),
            color::Fg(color::Reset)
        );
        let mut actual = String::new();

        write!(&mut actual, "{}", line).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn line_highlights_the_choice_matching_section_when_selected() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let window = Window::new(&terminal, input_len);
        let position = 0; // the current selection

        let choice = Choice::new("sample_file.rs".to_string(), 2, 8);

        let line = Line::new(choice, position, &window);

        let expected = format!(
            "{}sa{}mple_f{}ile.rs{}",
            style::Invert,
            color::Fg(color::LightGreen),
            color::Fg(color::Reset),
            style::Reset
        );
        let mut actual = String::new();

        write!(&mut actual, "{}", line).unwrap();

        assert_eq!(expected, actual);
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
