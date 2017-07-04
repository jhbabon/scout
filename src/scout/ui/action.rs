use termion::event::Key;

/// Represent all possible user's actions
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Action {
    /// Remove a char from the current query
    DeleteChar,
    /// Select the choice that is on top to the corrent selection
    MoveUp,
    /// Select the choice that is down the corrent selection
    MoveDown,
    /// The search is done
    Done,
    /// Clear the query string
    Clear,
    /// Add a new char to the current query string
    Add(char),
    /// Exit without selecting anything
    Exit,
}

impl Action {
    /// Get an action from an input key.
    ///
    /// You can get the same action with different keys (i.e: C-n and Arrow key down are the same
    /// action, MoveDown).
    ///
    /// NOTE: We don't use the From trait because we are returning an Option and TryFrom is still
    /// experimental.
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
