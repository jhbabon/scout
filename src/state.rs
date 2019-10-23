use rayon::prelude::*;
use std::collections::VecDeque;
use crate::fuzzy::Candidate;

#[derive(Debug,Clone,Default)]
pub struct State {
    pub query_string: String,
    pub matches: Vec<Candidate>,
    selection_idx: usize,
}

// Rendering
// state.candidates() => Current list of candidates
// state.query_string() => String
// state.total_len() => Total len of canidates
// state.matching_len() => Candidates len
// state.selection_idx() => Position of the selected candidate

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_up(&mut self) {
        if self.selection_idx == 0 {
            self.selection_idx = self.max_selection();
        } else {
            self.selection_idx -= 1;
        }
    }

    pub fn select_down(&mut self) {
        if self.selection_idx == self.max_selection() {
            self.selection_idx = 0;
        } else {
            self.selection_idx += 1;
        }
    }

    pub fn update_query_string(&mut self, q: String) {
        self.query_string = q;
    }

    pub fn query_string(&self) -> String {
        self.query_string.clone()
    }

    pub fn selection_idx(&self) -> usize {
        self.selection_idx
    }

    pub fn selection(&self) -> Option<String> {
        match self.matches.iter().nth(self.selection_idx) {
            Some(candidate) => Some(candidate.string.clone()),
            None => None,
        }
    }

    fn max_selection(&self) -> usize {
        let len = self.matches.len();

        if len == 0 {
            0
        } else {
            len - 1
        }
    }
}
