use log::debug;
use rayon::prelude::*;
use crate::fuzzy::Candidate;

#[derive(Debug,Clone, Default)]
pub struct State {
    pub query: Vec<char>,
    pub pool: Vec<String>,
    pub matches: Vec<Candidate>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_char(&mut self, ch: char) {
        self.query.push(ch);
    }

    pub fn add_string(&mut self, string: String) {
        self.pool.push(string);
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
