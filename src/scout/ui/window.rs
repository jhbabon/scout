use super::Action;
use terminal::Measurable;

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
        let actions = [Some(Action::MoveDown)];

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
        let actions = [Some(Action::MoveDown), Some(Action::MoveUp)];

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

        let actions = [Some(Action::MoveDown), Some(Action::MoveDown)];
        window.outline(&actions, choices_len);

        assert_eq!(0, window.selection());

        let actions = [Some(Action::MoveUp)];
        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
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

        let actions = [Some(Action::MoveDown), Some(Action::MoveDown)];
        window.outline(&actions, choices_len);

        assert_eq!(0, window.selection());

        let actions = [Some(Action::MoveUp)];
        window.outline(&actions, choices_len);

        assert_eq!(1, window.selection());
    }
}
