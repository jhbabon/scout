//! The state of the program including interactions (moving around), last query, search
//! results and current selection

use crate::common::{Prompt, Text};
use crate::fuzzy::Candidate;

/// Possible updates done to the State
#[derive(Debug, Clone)]
pub enum StateUpdate {
    /// The query has changed
    Query,
    /// Any other update
    All,
}

impl Default for StateUpdate {
    fn default() -> Self {
        Self::All
    }
}

/// Current state of the program
#[derive(Debug, Clone, Default)]
pub struct State {
    search: Option<Prompt>,
    matches: Vec<Candidate>,
    pool_len: usize,
    selection_idx: usize,
    last_update: StateUpdate,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_search(&mut self, search: Prompt) {
        self.search = Some(search);
        self.last_update = StateUpdate::Query;
    }

    pub fn query(&self) -> String {
        match &self.search {
            Some(sb) => sb.as_string(),
            None => "".into(),
        }
    }

    pub fn cursor_until_end(&self) -> usize {
        match &self.search {
            Some(sb) => sb.cursor_until_end(),
            None => 0,
        }
    }

    pub fn set_matches(&mut self, matches: (Vec<Candidate>, usize)) {
        self.matches = matches.0;
        self.pool_len = matches.1;

        if self.selection_idx >= self.max_selection() {
            self.selection_idx = self.max_selection();
        }

        self.last_update = StateUpdate::All;
    }

    pub fn matches(&self) -> &Vec<Candidate> {
        &self.matches
    }

    pub fn pool_len(&self) -> usize {
        self.pool_len
    }

    pub fn last_update(&self) -> &StateUpdate {
        &self.last_update
    }

    pub fn select_up(&mut self) {
        if self.selection_idx == 0 {
            self.selection_idx = self.max_selection();
        } else {
            self.selection_idx -= 1;
        }
        self.last_update = StateUpdate::All;
    }

    pub fn select_down(&mut self) {
        if self.selection_idx == self.max_selection() {
            self.selection_idx = 0;
        } else {
            self.selection_idx += 1;
        }
        self.last_update = StateUpdate::All;
    }

    pub fn selection_idx(&self) -> usize {
        self.selection_idx
    }

    pub fn selection(&self) -> Option<Text> {
        match self.matches.get(self.selection_idx) {
            Some(candidate) => Some(candidate.text.clone()),
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
