use log::debug;
use rayon::prelude::*;
use crate::fuzzy::Candidate;

#[derive(Debug,Clone, Default)]
pub struct State {
    pub query: Vec<char>,
    pub pool: Vec<String>,
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

    pub fn add_candidate(&mut self, string: String) {
        self.pool.push(string);
    }

    pub fn candidates_done(&mut self) {
        // NOOP
    }

    pub fn add_input(&mut self, ch: char) {
        self.selection_idx = 0;
        self.query.push(ch);
    }

    pub fn del_input(&mut self) {
        self.selection_idx = 0;
        let _ch = self.query.pop();
    }

    pub fn clear_query(&mut self) {
        self.selection_idx = 0;
        self.query = vec![];
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

    pub fn query_string(&self) -> String {
        self.query.iter().collect()
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

    // NOTE: This is just temporary, the search should
    // be outside the state
    pub fn search(&mut self) {
        let q = self.query.iter().collect::<String>();

        self.matches = self.pool
            .par_iter()
            .map(|s| Candidate::best_match(&q, &s))
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .inspect(|c| debug!("[State#search] Candidate: {:?}", c))
            .collect();

        self.matches.par_sort_unstable_by(|a, b| b.cmp(a));
    }
}
