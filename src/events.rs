//! Represent what is happening in the program.
//!
//! All tasks communicate between them using events.

use crate::common::{Pool, Prompt, Text};
use crate::fuzzy::Candidate;
use std::time::Instant;

#[derive(Clone, Debug)]
pub enum Event {
    /// New line from STDIN
    NewLine(String),
    /// Signal that STDIN is done
    EOF,

    /// Move selection up
    Up,
    /// Move selection down
    Down,
    /// Exit the program without selecting anything
    Exit,
    /// Exit with selection
    Done,

    /// Perform a new search
    Search(Prompt),
    /// Results from a search
    SearchDone((Vec<Candidate>, usize, Instant)),
    /// Flush the screen with the given list of candidates
    Flush((Vec<Candidate>, usize)),

    // Surroundings events
    Pool(Pool<Text>),
    // TODO: Use an Arc here?
    Surroundings(Candidate),
    SurroundingsDone((Vec<Text>, Vec<Text>)),
}
