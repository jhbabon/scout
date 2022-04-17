//! Represent what is happening in the program.
//!
//! All tasks communicate between them using events.

use crate::common::Prompt;
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
    /// New selection index
    Select(usize),

    /// Exit the program without selecting anything
    Exit,
    /// Exit with selection
    Done,

    /// Perform a new search
    Search(Prompt),
    /// Results from a search
    SearchDone((Vec<Candidate>, usize, usize, Instant)),
    /// Flush the screen with the given list of candidates
    Flush((Vec<Candidate>, usize, usize)),

    /// NO-OP. Used to make some internal streams happy
    Ignore,
}
