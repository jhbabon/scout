use super::Action;
use terminal::Measurable;

/// The main UI window.
///
/// It has the following information:
///
/// * The width of the prompt indicator (where the users types the query).
/// * The dimensions: width and height.
/// * The current selected choice.
#[derive(Clone, Copy, Default, Debug)]
pub struct Window {
    prompt_width: usize,
    width: usize,
    height: usize,
    selection: usize,
    offset: usize,
}

impl Window {
    /// Create a new Window in a Terminal and knowing the len of the initial list of choices
    pub fn new<T: Measurable>(terminal: &T, input_len: usize) -> Self {
        let prompt_width = format!("{}", input_len).len();
        let (width, height) = terminal.size();
        let selection = 0;
        let offset = 0;

        Self {
            prompt_width,
            width,
            height,
            selection,
            offset,
        }
    }

    /// Evaluate a set of actions and change the window properties based on them.
    ///
    /// Some `ui::Action` have the effect of changing the window items, like changing the current
    /// selection.
    pub fn outline(&mut self, actions: &[Action], choices_len: usize) {
        let max_position = if choices_len == 0 { 0 } else { choices_len - 1 };

        for action in actions {
            match *action {
                Action::MoveUp => self.move_up(max_position),
                Action::MoveDown => self.move_down(max_position),
                _ => self.reset(),
            }
        }

        self.scroll();
    }

    /// Move the selection up
    fn move_up(&mut self, max_position: usize) {
        if self.selection == 0 {
            self.selection = max_position;
        } else {
            self.selection -= 1;
        }
    }

    /// Move the selection down
    fn move_down(&mut self, max_position: usize) {
        if self.selection == max_position {
            self.selection = 0;
        } else {
            self.selection += 1;
        }
    }

    /// Move the window's offset based on the current selection
    fn scroll(&mut self) {
        let top_position = self.offset;
        let last_position = (self.lines_len() + self.offset) - 1;

        if self.selection > last_position {
            self.offset += self.selection - last_position;
        } else if self.selection < top_position {
            self.offset -= top_position - self.selection;
        }
    }

    fn reset(&mut self) {
        self.selection = 0;
        self.offset = 0;
    }

    /// Get the width of the prompt indicator of num of choices
    pub fn prompt_width(&self) -> usize {
        self.prompt_width
    }

    /// What is the index of the current selected Choice
    pub fn selection(&self) -> usize {
        self.selection
    }

    /// What is the offset of hidden choices in the current window
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the window width
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the window height
    pub fn height(&self) -> usize {
        self.height
    }

    /// How many lines of choices can display the window
    pub fn lines_len(&self) -> usize {
        if self.height() > 1 {
            self.height() - 2
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
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
    fn it_builds_with_correct_proportions() {
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
    fn its_lines_len_is_proportional_to_its_height() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 30,
        };

        let window = Window::new(&terminal, input_len);

        assert_eq!(28, window.lines_len());
    }

    #[test]
    fn its_lines_len_is_zero_on_small_terminals() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 1,
        };

        let window = Window::new(&terminal, input_len);

        assert_eq!(0, window.lines_len());
    }

    #[test]
    fn it_moves_the_selection_down_when_outlining() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let choices_len = 10;

        let mut window = Window::new(&terminal, input_len);
        let actions = [Action::MoveDown];

        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
    }

    #[test]
    fn it_moves_the_selection_back_up_when_outlining() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let choices_len = 10;

        let mut window = Window::new(&terminal, input_len);
        let actions = [Action::MoveDown, Action::MoveUp];

        window.outline(&actions, choices_len);

        assert_eq!(0, window.selection());
    }

    #[test]
    fn it_cycles_through_the_selections_when_outlining() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 4,
        };
        let choices_len = 10;

        let mut window = Window::new(&terminal, input_len);

        let actions = [Action::MoveDown, Action::MoveDown];
        window.outline(&actions, choices_len);

        assert_eq!(2, window.selection());
        assert_eq!(1, window.offset());

        let actions = [Action::MoveUp];
        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
        assert_eq!(1, window.offset());
    }

    #[test]
    fn it_cycles_through_the_selections_when_outlining_using_choices_len_when_necessary() {
        let input_len = 10;
        let terminal = DummyTerminal {
            width: 100,
            height: 40,
        };
        let choices_len = 2;

        let mut window = Window::new(&terminal, input_len);

        let actions = [Action::MoveDown, Action::MoveDown];
        window.outline(&actions, choices_len);

        assert_eq!(0, window.selection());
        assert_eq!(0, window.offset());

        let actions = [Action::MoveUp];
        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
        assert_eq!(0, window.offset());
    }
}
