// I don't feel like I can build a good fuzzy search algorithm
// so let's use a library, at least for the moment
use crate::common::Text;
use async_std::sync::Arc;
use std::cmp::Ordering;
use sublime_fuzzy::{best_match, Match};

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub text: Text,
    pub score_match: Option<Match>,
}

impl Candidate {
    pub fn new(text: String) -> Self {
        Self {
            text: Arc::new(text),
            score_match: None,
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

pub fn finder(query: &str, target: Text) -> Option<Candidate> {
    if query.is_empty() {
        let candidate = Candidate {
            text: target,
            score_match: None,
        };
        return Some(candidate);
    }

    match best_match(query, &target) {
        None => None,
        Some(score_match) => {
            let candidate = Candidate {
                text: target,
                score_match: Some(score_match),
            };
            Some(candidate)
        }
    }
}

// =======================================================================
// Let's try to implement fuzzaldrin-plus algorithm
// @see: https://github.com/jeancroy/fuzz-aldrin-plus/blob/84eac1d73bacbbd11978e6960f4aa89f8396c540/src/scorer.coffee#L83
// =======================================================================

#[derive(Debug, Clone)]
pub struct Query {
    pub string: String,
    pub graphemes: Vec<String>,
    pub graphemes_lw: Vec<String>,
}

impl Query {
    pub fn new(string: String) -> Self {
        let graphemes = string.graphemes(true).map(|s| String::from(s)).collect::<Vec<_>>();
        let graphemes_lw = string.to_lowercase().graphemes(true).map(|s| String::from(s)).collect::<Vec<_>>();

        Self {
            string,
            graphemes,
            graphemes_lw,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }
}

impl From<&str> for Query {
    fn from(string: &str) -> Self {
        Self::new(String::from(string))
    }
}

// Candidate replacement. This represent a possible choice
#[derive(Debug, Clone)]
pub struct Subject {
    pub text: Text,
    pub graphemes: Arc<Vec<String>>,
    pub graphemes_lw: Arc<Vec<String>>,
    pub score: f32,
    pub matches: Vec<usize>,
}

impl Subject {
    pub fn new(string: String) -> Self {
        let text = Arc::new(string);
        let graphemes = Arc::new(text.graphemes(true).map(|s| String::from(s)).collect::<Vec<_>>());
        let graphemes_lw = Arc::new(text.to_lowercase().graphemes(true).map(|s| String::from(s)).collect::<Vec<_>>());
        let score = 0.0;
        let matches = Vec::new();

        Self {
            text,
            graphemes,
            graphemes_lw,
            score,
            matches,
        }
    }
}

impl From<&Subject> for Subject {
    fn from(other: &Subject) -> Self {
        let text = other.text.clone();
        let graphemes = other.graphemes.clone();
        let graphemes_lw = other.graphemes_lw.clone();
        let score = 0.0;
        let matches = Vec::new();

        Self {
            text,
            graphemes,
            graphemes_lw,
            score,
            matches,
        }
    }
}

impl From<&str> for Subject {
    fn from(string: &str) -> Self {
        Self::new(String::from(string))
    }
}

// For the moment let's assume the query is always in lowercase
pub fn score(query: &Query, subject: &Subject) -> Option<Subject> {
    if query.is_empty() {
        let new_subject = Subject::from(subject);

        return Some(new_subject);
    }

    // -----------------------------------------------------------------
    // Check if the query is inside the subject
    if !is_match(query, subject) {
        return None;
    }

    // -----------------------------------------------------------------
    // TODO: Abbreviations sequence

    // -----------------------------------------------------------------
    // TODO: Exact Match

    // -----------------------------------------------------------------
    // TODO: Individual characters
    // (Smith Waterman algorithm)

    let new_subject = Subject::from(subject);

    Some(new_subject)
}

fn is_match(query: &Query, subject: &Subject) -> bool {
    let subject_count = subject.graphemes_lw.len();

    let mut query_iter = query.graphemes_lw.iter();
    let mut subject_iter = subject.graphemes_lw.iter();

    let mut count = 0;
    let mut done = false;
    while let Some(query_char) = query_iter.next() {
        if done {
            // The subject_chars collection is done, but not the query_chars
            // which means that the query is not inside the subject text
            return false;
        }

        'inner: while let Some(subject_char) = subject_iter.next() {
            count += 1;
            if query_char == subject_char {
                break 'inner;
            }
        }

        if count == subject_count {
            done = true;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_new_subject_on_empty_queries() {
        let query = Query::from("");
        let subject = Subject::from("Foo");

        let result = score(&query, &subject);

        assert!(result.is_some());

        let prt = result.unwrap();

        assert_eq!(prt.text, subject.text);
        assert_eq!(prt.score, 0.0);
        assert!(prt.matches.is_empty());
    }

    #[test]
    fn it_returns_none_if_the_query_is_bigger_than_the_text() {
        let query = Query::from("bar");
        let subject = Subject::from("Ba");

        let result = score(&query, &subject);

        assert!(result.is_none());
    }

    #[test]
    fn it_returns_none_if_the_query_is_not_inside_the_text() {
        let query = Query::from("bar");
        let subject = Subject::from("Foo");

        let result = score(&query, &subject);

        assert!(result.is_none());
    }

    #[test]
    fn it_returns_some_if_the_query_is_inside_the_text() {
        let query = Query::from("bar");
        let texts = vec!["Bar", "Fboaor"];
        for text in texts {
            let subject = Subject::from(text);

            let result = score(&query, &subject);

            assert!(result.is_some());
        }
    }
}
