use termion::event::Key;

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
