use termion::event::Key;

#[derive(Clone, Debug)]
pub enum Event {
    Packet(String), // New input from main input
    EOF,            // EOF of main input
    Input(char),    // New input char
    Up,             // Go up
    Down,           // Go down
    Backspace,      // Delete char
    Clear,          // Clear query string
    Exit,           // Exit without selecting anything
    Done,           // Exit with selection
    Ignore,         // NO-OP
}

impl From<Key> for Event {
    fn from(key: Key) -> Self {
        match key {
            Key::Ctrl('u') => Event::Clear,
            Key::Ctrl('p') | Key::Up => Event::Up,
            Key::Ctrl('n') | Key::Down => Event::Down,

            Key::Esc | Key::Alt('\u{0}') => Event::Exit,
            Key::Backspace => Event::Backspace,
            Key::Char('\n') => Event::Done,

            Key::Char(ch) => Event::Input(ch),

            _ => Event::Ignore,
        }
    }
}
