use crate::fuzzy::Candidate;

#[derive(Clone, Debug)]
pub enum Event {
    Packet(String), // New input from main input
    EOF,            // EOF of main input
    Up,             // Go up
    Down,           // Go down
    Exit,           // Exit without selecting anything
    Done,           // Exit with selection
    Ignore,         // NO-OP
    Query(String),
    Matches((Vec<Candidate>, usize)),
}
