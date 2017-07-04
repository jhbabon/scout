use termion::{color, style};

use std::fmt;
use choice::Choice;
use super::Window;

/// Represent a line in the list of choices.
///
/// A line highlights the matching section of a choice and is different in the case the choice is
/// the current user's selection.
pub struct Line {
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
    fn it_highlights_the_choice_matching_section() {
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
    fn it_highlights_the_choice_matching_section_on_small_width() {
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
    fn it_highlights_the_choice_matching_section_when_selected() {
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
}
