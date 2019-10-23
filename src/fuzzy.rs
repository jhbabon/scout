// I don't feel like I can build a good fuzzy search algorithm
// so let's use a library, at least for the moment
use sublime_fuzzy::{best_match, Match};
use std::cmp::Ordering;

#[derive(Debug,Clone)]
pub struct Candidate {
    pub string: String,
    pub score_match: Option<Match>,
}

impl Candidate {
    pub fn new(string: String) -> Self {
        Self { string, score_match: None }
    }

    pub fn best_match(query: &str, target: &str) -> Option<Self> {
        if query.is_empty() {
            let candidate = Self { string: target.to_string(), score_match: None };
            return Some(candidate);
        }

        match best_match(query, target) {
            None => None,
            Some(score_match) => {
                let candidate = Self {
                    string: target.to_string(),
                    score_match: Some(score_match),
                };
                Some(candidate)
            },
        }
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Candidate) -> Ordering {
        self.score_match.cmp(&other.score_match)
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Candidate) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Candidate {}

impl PartialEq for Candidate {
    fn eq(&self, other: &Candidate) -> bool {
        self.score_match == other.score_match
    }
}
