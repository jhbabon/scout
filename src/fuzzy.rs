pub mod types;
mod predicates;
mod scoring;

use types::*;
use predicates::*;
use scoring::*;

use crate::common::Text;
use async_std::sync::Arc;
use std::cmp::Ordering;
use sublime_fuzzy::{best_match, Match};


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
// Max number missed consecutive hit = ceil(MISS_COEFF * query.len) + 5
const MISS_COEFF: f32 = 0.75;

// probably is better to use something like {Score|Scoring}<Subject> instead of overloading Subject
// with score and matched fields
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
    // Acronym sequence
    let acronym = score_acronyms(query, subject);

    // The whole query is an acronym, let's return that
    if acronym.count == query.len {
        let score = score_quality(query.len, subject.len, acronym.score, acronym.position);

        let mut new_subject = Subject::from(subject);
        new_subject.score = score;

        return Some(new_subject);
    }

    // -----------------------------------------------------------------
    // Exact Match
    if let Some(score) = score_exact_match(query, subject) {
        let mut new_subject = Subject::from(subject);
        new_subject.score = score;

        return Some(new_subject);
    }

    // -----------------------------------------------------------------
    // Individual characters
    // (Smith Waterman algorithm)


    let mut new_subject = Subject::from(subject);
    new_subject.score = acronym.score;

    Some(new_subject)
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
        let candidates = vec!["Bar", "Fboaor"];
        for candidate in candidates {
            let subject = Subject::from(candidate);

            let result = score(&query, &subject);

            assert!(result.is_some());
        }
    }

    #[test]
    fn it_returns_acronym_scores() {
        let query = Query::from("fft");

        let subject_a = Subject::from("FirstFactoryTests.html");
        let subject_b = Subject::from("FirstFactory.html");

        let result_a = score(&query, &subject_a);
        let result_b = score(&query, &subject_b);

        assert!(result_a.is_some());
        assert!(result_b.is_some());

        let result_a = result_a.unwrap();
        let result_b = result_b.unwrap();
        let score_a = result_a.score;
        let score_b = result_b.score;
        assert!(
            score_a > score_b,
            "Expected score {:?} from {:?} to be greater than {:?} from {:?}",
            result_a.score,
            result_a.text,
            result_b.score,
            result_b.text
        );
    }

    #[test]
    fn it_returns_exact_match_scores() {
        let query = Query::from("core");

        let subject_a = Subject::from("parser_core.rs");
        let subject_b = Subject::from("somethingcorexcore");

        let result_a = score(&query, &subject_a);
        let result_b = score(&query, &subject_b);

        assert!(result_a.is_some());
        assert!(result_b.is_some());

        let result_a = result_a.unwrap();
        let result_b = result_b.unwrap();
        let score_a = result_a.score;
        let score_b = result_b.score;
        assert!(
            score_a > score_b,
            "Expected score {:?} from {:?} to be greater than {:?} from {:?}",
            result_a.score,
            result_a.text,
            result_b.score,
            result_b.text
        );
    }
}
