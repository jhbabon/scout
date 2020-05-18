use crate::common::Prompt;
use crate::fuzzy::Candidate;
use std::time::Instant;

#[derive(Clone, Debug)]
pub enum Event {
    NewLine(String), // New line from main input
    EOF,             // EOF of main input

    Up,     // Go up
    Down,   // Go down
    Exit,   // Exit without selecting anything
    Done,   // Exit with selection
    Ignore, // NO-OP

    Search(Prompt),
    SearchDone((Vec<Candidate>, usize, Instant)),
    Flush((Vec<Candidate>, usize)),
}
