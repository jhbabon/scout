mod predicates;
mod scoring;
mod types;

use predicates::*;
use scoring::*;
use types::*;

use crate::common::Text;
use rayon::prelude::*;
use std::collections::VecDeque;

pub use types::{Candidate, Query};

// Max number missed consecutive hit = ceil(MISS_COEFF * query.len()) + 5
const MISS_COEFF: f32 = 0.75;

/// Search for candidates to match a query
///
/// * If the query is empty it just returns the same pool of candidates
/// * Otherwise it will try to compute the best match for each candidate
///   and then sort them from higher score to lower
pub fn search(q: &str, pool: &VecDeque<Text>) -> Vec<Candidate> {
    let mut matches: Vec<Candidate>;
    let query: Query = q.into();

    if query.is_empty() {
        matches = pool.par_iter().map(|txt| txt.into()).collect();
    } else {
        matches = pool
            .par_iter()
            .map(|c| compute_match(&query, &c))
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .collect();

        matches.par_sort_unstable_by(|a, b| b.cmp(a));
    }

    matches
}

/// Custom port of the fuzzaldrin-plus algorithm used in Atom editor.
///
/// This function will return a Candidate with the computed score and
/// matches
///
/// NOTE: The only part missing (I think) from the original algorithm
///   is the path score bonus
///
/// Links:
///   * https://github.com/jeancroy/fuzz-aldrin-plus/blob/84eac1d73bacbbd11978e6960f4aa89f8396c540/src/scorer.coffee#L83
///   * https://github.com/jeancroy/fuzz-aldrin-plus/blob/84eac1d73bacbbd11978e6960f4aa89f8396c540/src/matcher.coffee#L172
fn compute_match(query: &Query, subject: &Text) -> Option<Candidate> {
    if query.is_empty() {
        return None;
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
    if acronym.count == query.len() {
        let score = score_quality(query.len(), subject.len(), acronym.score, acronym.position);
        let matches = acronym.matches;

        return Some(Candidate::new(subject, score, matches));
    }

    // -----------------------------------------------------------------
    // Exact Match
    if let Some(result) = score_exact_match(query, subject) {
        let score = result.score;
        let matches = result.matches;

        return Some(Candidate::new(subject, score, matches));
    }

    // -----------------------------------------------------------------
    // Individual characters
    // (Smith Waterman algorithm)

    // Init
    let mut score;
    let mut score_row = vec![0.0_f32; query.len()];
    let mut consecutive_row = vec![0.0_f32; query.len()];
    let scored_size = score_size(query.len(), subject.len());

    let miss_budget = (MISS_COEFF * query.len() as f32).ceil() + 5.0;
    let mut miss_left = miss_budget;
    let mut should_rebuild = true;

    // Trace Matrix, this is used to recover best matches positions
    let mut trace = TraceMatrix::new(subject.len(), query.len());

    let mut subject_iter = subject.lowercase_iter().enumerate();
    'subject_loop: while let Some((subject_index, subject_grapheme)) = subject_iter.next() {
        if !query.contains(subject_grapheme) {
            if should_rebuild {
                consecutive_row = vec![0.0_f32; query.len()];
                should_rebuild = false;
            }

            continue 'subject_loop;
        }

        score = 0.0;
        let mut score_diag = 0.0;
        let mut consecutive_diag = 0.0;
        let mut record_miss = true;
        should_rebuild = true;

        let mut query_iter = query.lowercase_iter().enumerate();
        while let Some((query_index, query_grapheme)) = query_iter.next() {
            let mut consecutive_score = 0.0;

            let score_up = score_row[query_index];
            if score_up >= score {
                score = score_up;
                trace.up_at(query_index, subject_index);
            } else {
                trace.left_at(query_index, subject_index);
            }

            if query_grapheme == subject_grapheme {
                let is_start = is_start_of_word(subject, subject_index);

                if consecutive_diag > 0.0 {
                    consecutive_score = consecutive_diag;
                } else {
                    consecutive_score =
                        score_consecutives(query, subject, query_index, subject_index, is_start);
                }

                let score_char =
                    score_character(subject_index, is_start, acronym.score, consecutive_score);
                let align = score_diag + score_char;

                if align > score {
                    score = align;
                    trace.diagonal_at(query_index, subject_index);
                    miss_left = miss_budget;
                } else {
                    consecutive_score = 0.0;

                    if record_miss {
                        miss_left -= 1.0;

                        if miss_left <= 0.0 {
                            let final_score =
                                score.max(score_row[query.last_index()]) * scored_size;
                            if final_score <= 0.0 {
                                return None;
                            } else {
                                let matches = trace.traceback(query_index, subject_index);

                                return Some(Candidate::new(subject, final_score, matches));
                            }
                        }
                    }

                    record_miss = false;
                }
            }

            score_diag = score_up;
            consecutive_diag = consecutive_row[query_index];
            consecutive_row[query_index] = consecutive_score;
            score_row[query_index] = score;

            if score <= 0.0 {
                trace.stop_at(query_index, subject_index);
            }
        }
    }

    let final_score = score_row[query.last_index()] * scored_size;
    if final_score == 0.0 {
        None
    } else {
        let matches = trace.traceback(query.last_index(), subject.last_index());

        Some(Candidate::new(subject, final_score, matches))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::TextBuilder;

    #[test]
    fn matches_test() {
        let cases: Vec<(Query, &str, Vec<usize>)> = vec![
            // Exact acronym
            ("fft".into(), "FirstFactoryTest", vec![0, 5, 12]),
            // Extra acronym letters
            ("fft".into(), "FirstFactoryTest.ts", vec![0, 5, 12]),
            // Exact match
            ("core".into(), "0core0app.rb", vec![1, 2, 3, 4]),
            // Exact match, second position is better
            (
                "core".into(),
                "0core0app_core.rb".into(),
                vec![10, 11, 12, 13],
            ),
            // Consecutive letters
            ("core".into(), "controller".into(), vec![0, 1, 4, 8]),
        ];

        for (query, string, expected) in cases {
            let subject = TextBuilder::build(string);
            let result = compute_match(&query, &subject);
            assert!(result.is_some());

            let result = result.unwrap();
            assert_eq!(
                result.matches, expected,
                "Expected {} to have matches {:?} inside {}",
                query, expected, subject,
            );
        }
    }

    #[test]
    fn compute_match_on_different_queries_test() {
        let cases: Vec<(Query, Query, &str)> = vec![
            // Acronym wins
            ("psh".into(), "push".into(), "Plus: Stage Hunk"),
            // Exact world wins
            ("Hello".into(), "he".into(), "Hello World"),
            // More consecutive chars wins
            ("ello".into(), "hllo".into(), "Hello World"),
        ];

        for (a, b, string) in cases {
            let subject = TextBuilder::build(string);
            let result_a = compute_match(&a, &subject);
            let result_b = compute_match(&b, &subject);

            assert!(result_a.is_some());
            assert!(result_b.is_some());

            let result_a = result_a.unwrap();
            let result_b = result_b.unwrap();

            assert!(
                result_a.score() > result_b.score(),
                "Expected {} to have a higher score than {} inside {}",
                a,
                b,
                subject,
            );
        }
    }
}
