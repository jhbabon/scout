use crate::common::SearchBox;
use crate::fuzzy::Candidate;
use std::time::Instant;

#[derive(Clone, Debug)]
pub enum Event {
    Packet(String), // New line from main input
    EOF,            // EOF of main input
    Up,             // Go up
    Down,           // Go down
    Exit,           // Exit without selecting anything
    Done,           // Exit with selection
    Ignore,         // NO-OP

    // TODO: Better names
    Request(SearchBox),

    Search((Vec<Candidate>, usize, Instant)),
    FlushSearch((Vec<Candidate>, usize)),
}
