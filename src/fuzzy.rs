mod predicates;
mod scoring;
pub mod types;

use log::debug;

use predicates::*;
use scoring::*;
use types::*;

use crate::common::Text;
use async_std::sync::Arc;
use std::cmp::Ordering;
use std::collections::HashSet;
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
    debug!("------------------------------------------------------------");
    debug!(
        "Got Query {:?} and Subject {:?}",
        query.string, subject.text
    );
    debug!(
        "query_len {:?} and subject_len {:?}",
        query.len, subject.len
    );

    if query.is_empty() {
        debug!("Empty query, returning None");
        return None;
    }

    // -----------------------------------------------------------------
    // Check if the query is inside the subject
    if !is_match(query, subject) {
        debug!("Query is not inside Subject, returning None");
        return None;
    }

    // -----------------------------------------------------------------
    // Acronym sequence
    let acronym = score_acronyms(query, subject);

    // The whole query is an acronym, let's return that
    if acronym.count == query.len {
        let score = score_quality(query.len, subject.len, acronym.score, acronym.position);
        debug!("Query is a full acronym, returning the score {:?}", score);

        let mut new_subject: Subject = subject.into();
        new_subject.score = score;

        return Some(new_subject);
    }

    // -----------------------------------------------------------------
    // Exact Match
    if let Some(score) = score_exact_match(query, subject) {
        debug!("Query is a exact match, returning the score {:?}", score);
        let mut new_subject: Subject = subject.into();
        new_subject.score = score;

        return Some(new_subject);
    }

    // -----------------------------------------------------------------
    // Individual characters
    // (Smith Waterman algorithm)

    // Init
    let mut score;
    let mut score_row = vec![0.0_f32; query.len];
    let mut consecutive_row = vec![0.0_f32; query.len];
    let scored_size = score_size(query.len, subject.len);

    debug!("scored_size {:?}", scored_size);

    // TODO: Move to Query struct
    let query_set: HashSet<String> = query.graphemes_lw.iter().map(|s| s.clone()).collect();

    let miss_budget = (MISS_COEFF * query.len as f32).ceil() + 5.0;
    let mut miss_left = miss_budget;
    let mut should_rebuild = true;

    let mut subject_index = 0;
    'subject_loop: while subject_index < subject.len {
        let subject_grapheme = &subject.graphemes_lw[subject_index];
        debug!(
            "  Checking grapheme {:?} at {:?}",
            subject_grapheme, subject_index
        );

        if !query_set.contains(subject_grapheme) {
            debug!("  It's not in Query");
            if should_rebuild {
                debug!("  Rebuilding");
                consecutive_row = vec![0.0_f32; query.len];
                should_rebuild = false;
            }

            subject_index += 1;
            continue 'subject_loop;
        }

        debug!("  It's in query");

        score = 0.0;
        let mut score_diag = 0.0;
        let mut consecutive_diag = 0.0;
        let mut record_miss = true;
        should_rebuild = true;

        let mut query_index = 0;
        while query_index < query.len {
            debug!("    Checking query at {:?}", query_index);

            let score_up = score_row[query_index];
            debug!(
                "    Scores so far: score_up {:?}, score {:?}",
                score_up, score
            );
            if score_up > score {
                debug!("    score is now score_up");
                score = score_up;
            }

            let mut consecutive_score = 0.0;

            if &query.graphemes_lw[query_index] == subject_grapheme {
                debug!("    It's a match!");

                let start = is_start_of_word(subject, subject_index);

                debug!("    Is a start of word? {:?}", start);

                debug!("    consecutive_diag {:?}", consecutive_diag);

                if consecutive_diag > 0.0 {
                    consecutive_score = consecutive_diag;
                } else {
                    consecutive_score =
                        score_consecutives(query, subject, query_index, subject_index, start);
                }

                debug!("    consecutive_score {:?}", consecutive_score);

                debug!("    score_diag {:?}", score_diag);
                let score_char =
                    score_character(subject_index, start, acronym.score, consecutive_score);
                debug!("    score_char {:?}", score_char);
                let align = score_diag + score_char;

                debug!("    align {:?}", align);

                if align > score {
                    debug!("    score is now align");
                    score = align;
                    miss_left = miss_budget;
                } else {
                    if record_miss {
                        debug!("    Recording a miss");
                        miss_left -= 1.0;

                        debug!("    miss_left {:?}", miss_left);

                        if miss_left <= 0.0 {
                            debug!("    No more miss_left budget");

                            let final_score = score.max(score_row[query.len - 1]) * scored_size;
                            debug!("    final_score after misses exhausted {:?}", final_score);
                            if final_score == 0.0 {
                                debug!("    Returning None");
                                return None;
                            } else {
                                let mut new_subject: Subject = subject.into();
                                new_subject.score = final_score;

                                return Some(new_subject);
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

            query_index += 1;
            debug!("    Next query_index {:?}", query_index);
        }

        subject_index += 1;
        debug!("  Next subject_index {:?}", subject_index);
    }

    debug!("Subject done");

    let final_score = score_row[query.len - 1] * scored_size;
    debug!("final_score {:?}", final_score);
    debug!("------------------------------------------------------------");
    if final_score == 0.0 {
        None
    } else {
        let mut new_subject: Subject = subject.into();
        new_subject.score = final_score;

        Some(new_subject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_when_there_are_no_results_test() {
        let cases: Vec<(Query, Subject)> = vec![
            ("".into(), "foo".into()),
            ("bar".into(), "ba".into()),
            ("bar".into(), "foo".into()),
        ];

        for (query, subject) in cases {
            assert!(
                score(&query, &subject).is_none(),
                "Expected {:?} to not be scored in {:?}",
                query.string,
                subject.text,
            );
        }
    }

    #[test]
    fn score_on_different_queries_test() {
        let cases: Vec<(Query, Query, Subject)> = vec![
            // Acronym wins
            ("psh".into(), "push".into(), "Plus: Stage Hunk".into()),
            // Exact world wins
            ("Hello".into(), "he".into(), "Hello World".into()),
            // More consecutive chars wins
            ("ello".into(), "hllo".into(), "Hello World".into()),
        ];

        for (a, b, subject) in cases {
            let result_a = score(&a, &subject);
            let result_b = score(&b, &subject);

            assert!(result_a.is_some());
            assert!(result_b.is_some());

            let result_a = result_a.unwrap();
            let result_b = result_b.unwrap();

            assert!(
                result_a.score > result_b.score,
                "Expected {:?} to have a higher score than {:?} inside {:?}",
                a.string,
                b.string,
                subject.text,
            );
        }
    }

    fn assert_scores_between_subjects(query: Query, a: Subject, b: Subject) {
        let result_a = score(&query, &a);
        let result_b = score(&query, &b);

        assert!(result_a.is_some());
        assert!(result_b.is_some());

        let result_a = result_a.unwrap();
        let result_b = result_b.unwrap();

        assert!(
            result_a.score > result_b.score,
            "Expected {:?} to have a higher score in {:?} than in {:?}. A {:?} <= B {:?}",
            query.string,
            a.text,
            b.text,
            result_a.score,
            result_b.score,
        );
    }

    #[test]
    fn score_on_exact_match_test() {
        assert_scores_between_subjects("file".into(), "Cargofile".into(), "filter".into());
    }

    #[test]
    fn score_on_extact_match_end_word_boundaries_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // End of world bonus (string limit)
            ("file".into(), "0cargofile".into(), "cargofile0".into()),
            // End of world bonus (separator limit)
            (
                "file".into(),
                "0cargofile world".into(),
                "hello cargofile0".into(),
            ),
            // End of world bonus (camelCase limit)
            (
                "file".into(),
                "0cargofileWorld".into(),
                "helloCargofile0".into(),
            ),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_extact_match_start_word_boundaries_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // Start of world bonus (string limit)
            ("cargo".into(), "cargofile0".into(), "0cargofile".into()),
            // Start of world bonus (separator limit)
            (
                "cargo".into(),
                "hello cargofile0".into(),
                "0cargofile world".into(),
            ),
            // Start of world bonus (camelCase limit)
            (
                "cargo".into(),
                "helloCargofile0".into(),
                "0cargofileWorld".into(),
            ),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_exact_match_preference_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // full-word > start-of-word
            ("core".into(), "0_core_000 x".into(), "0_core0_00 x".into()),
            // start-of-word > end-of-word
            ("core".into(), "0_core0_00 x".into(), "0core_0000 x".into()),
            // end-of-word > middle-of-word
            ("core".into(), "0core_0000 x".into(), "0core0_000 x".into()),
            // middle-of-word > split
            ("core".into(), "0core0_000 x".into(), "0_co_re_00 x".into()),
            // split > scattered letters
            ("core".into(), "0_co_re_00 x".into(), "controller x".into()),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_exact_match_with_multi_word_preference_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // full-word > start-of-word
            (
                "core x".into(),
                "0_core_000 x".into(),
                "0_core0_00 x".into(),
            ),
            // start-of-word > end-of-word
            (
                "core x".into(),
                "0_core0_00 x".into(),
                "0core_0000 x".into(),
            ),
            // end-of-word > middle-of-word
            (
                "core x".into(),
                "0core_0000 x".into(),
                "0core0_000 x".into(),
            ),
            // middle-of-word > split
            (
                "core x".into(),
                "0core0_000 x".into(),
                "0_co_re_00 x".into(),
            ),
            // split > scattered letters
            (
                "core x".into(),
                "0_co_re_00 x".into(),
                "controller x".into(),
            ),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_exact_match_case_insensitive_over_complete_word_test() {
        assert_scores_between_subjects("file".into(), "ZFILEZ".into(), "fil e".into());
    }

    #[test]
    fn score_on_exact_match_prefer_smaller_haystack_test() {
        assert_scores_between_subjects("core".into(), "core".into(), "core_".into());
    }

    #[test]
    fn score_on_exact_match_prefer_match_at_start_of_string_test() {
        assert_scores_between_subjects("core".into(), "core_data".into(), "data_core".into());
        assert_scores_between_subjects(
            "core".into(),
            "hello_core_data".into(),
            "hello_data_core".into(),
        );
    }

    #[test]
    fn score_on_exact_match_prefer_single_letter_start_of_world_test() {
        assert_scores_between_subjects(
            "m".into(),
            "Markdown Preview: Copy Html".into(),
            "Timecop: View".into(),
        );
        assert_scores_between_subjects(
            "m".into(),
            "Markdown Preview: Toggle Break On Newline".into(),
            "Welcome: Show".into(),
        );
        assert_scores_between_subjects("d".into(), "doc/REAME".into(), "TODO".into());
    }

    #[test]
    fn score_on_exact_match_selects_better_occurences_test() {
        assert_scores_between_subjects("es".into(), "Test Español".into(), "Portugues".into());
    }

    #[test]
    fn score_on_consecutive_letters_preference_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // full-word > start-of-word
            ("modelcore".into(), "model-0-core-000.x".into(), "model-0-core0-00.x".into()),
            // start-of-word > end-of-word
            ("modelcore".into(), "model-0-core0-00.x".into(), "model-0core-0000.x".into()),
            // end-of-word > middle-of-word
            ("modelcore".into(), "model-0core-0000.x".into(), "model-0core0-000.x".into()),
            // middle-of-word > scattered letters
            ("modelcore".into(), "model-0core0-000.x".into(), "model-controller.x".into()),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_consecutive_letters_full_word_preference_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // full-word > start-of-word
            ("modelcorex".into(), "model-0-core-000.x".into(), "model-0-core0-00.x".into()),
            // start-of-word > end-of-word
            ("modelcorex".into(), "model-0-core0-00.x".into(), "model-0core-0000.x".into()),
            // end-of-word > middle-of-word
            ("modelcorex".into(), "model-0core-0000.x".into(), "model-0core0-000.x".into()),
            // middle-of-word > scattered letters
            ("modelcorex".into(), "model-0core0-000.x".into(), "model-controller.x".into()),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_consecutive_letters_preference_test_vs_directory_depth_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // full-word > start-of-word
            ("model core".into(), "0/0/0/0/model/core_000.x".into(), "0/0/0/model/core0_00.x".into()),
            // start-of-word > end-of-word
            ("model core".into(), "0/0/0/model/core0_00.x".into(), "0/0/model/0core_00.x".into()),
            // end-of-word > middle-of-word
            ("model core".into(), "0/0/model/0core_00.x".into(), "0/model/0core0_0.x".into()),
            // middle-of-word > scattered letters
            ("model core".into(), "0/model/0core0_0.x".into(), "model/controller.x".into()),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_consecutive_letters_score_higher_than_scattered_test() {
        assert_scores_between_subjects("acon".into(), "applicatio_controller.rb".into(), "application.rb".into());
    }

    #[test]
    fn score_prefers_larger_groups_of_consecutive_letters_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            ("abcdef".into(), "  abcdef".into(), " abcde f".into()),
            ("abcdef".into(), " abcde f".into(), " abcd ef".into()),
            ("abcdef".into(), " abcd ef".into(), " abc def".into()),
            ("abcdef".into(), " abc def".into(), "ab cd ef".into()),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_prefers_larger_group_of_consecutive_letters_vs_better_context_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // 2 x 3 vs 3 x 2
            ("abcdef".into(), "0abc0def0".into(), "ab cd ef".into()),
            // 1 x 4 + 2 vs 2 x 2 + 2
            ("abcdef".into(), "0abcd0ef0".into(), "ab cd ef".into()),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_allows_consecutive_letter_in_path_overcome_deeper_path_test() {
        assert_scores_between_subjects("core app".into(), "controller/core/app.rb".into(), "controller/app.rb".into());
    }

    #[test]
    fn score_weighs_matches_at_the_start_of_the_string_or_base_name_higher_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            ("ab".into(), "a_b".into(), "a_b_c".into()),
            ("ab".into(), "a_b".into(), "z_a_b".into()),
            ("ab".into(), "a_b_c".into(), "c_a_b".into()),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    // TODO: Acronym + Case Sensitivity tests
}
