mod predicates;
mod scoring;
mod types;

use predicates::*;
use scoring::*;
use types::*;

use crate::common::Text;

// TODO: Replace Subject with Candidate instead of using a type (alias)
pub type Candidate = Subject;
pub use types::Query;

// =======================================================================
// Let's try to implement fuzzaldrin-plus algorithm
// @see: https://github.com/jeancroy/fuzz-aldrin-plus/blob/84eac1d73bacbbd11978e6960f4aa89f8396c540/src/scorer.coffee#L83
// =======================================================================
// Max number missed consecutive hit = ceil(MISS_COEFF * query.len()) + 5
const MISS_COEFF: f32 = 0.75;

pub fn compute(query: &Query, text: &Text) -> Option<Subject> {
    let subject = text.into();
    score(&query, &subject)
}

// TODO: Remove score an leave only compute. Update tests to use Text instead of Subject
fn score(query: &Query, subject: &Subject) -> Option<Subject> {
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

        return Some(subject.refine(score, matches));
    }

    // -----------------------------------------------------------------
    // Exact Match
    if let Some(result) = score_exact_match(query, subject) {
        let score = result.score;
        let matches = result.matches;

        return Some(subject.refine(score, matches));
    }

    // -----------------------------------------------------------------
    // Individual characters
    // (Smith Waterman algorithm)

    // Init
    let mut score;
    let mut score_row = vec![0.0_f32; query.len()];
    let mut consecutive_row = vec![0.0_f32; query.len()];
    let scored_size = score_size(query.len(), subject.len());

    // backtrace Matrix
    let mut trace = TraceMatrix::new(subject.len(), query.len());

    let miss_budget = (MISS_COEFF * query.len() as f32).ceil() + 5.0;
    let mut miss_left = miss_budget;
    let mut should_rebuild = true;

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
            let score_up = score_row[query_index];
            if score_up >= score {
                score = score_up;
                trace.up_at(query_index, subject_index);
            } else {
                trace.left_at(query_index, subject_index);
            }

            let mut consecutive_score = 0.0;

            if query_grapheme == subject_grapheme {
                let is_start = is_start_of_word(subject, subject_index);

                if consecutive_diag > 0.0 {
                    consecutive_score = consecutive_diag;
                } else {
                    consecutive_score =
                        score_consecutives(query, subject, query_index, subject_index, is_start);
                }

                let align = score_diag
                    + score_character(subject_index, is_start, acronym.score, consecutive_score);

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

                                return Some(subject.refine(final_score, matches));
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

        Some(subject.refine(final_score, matches))
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
                "Expected {} to not be scored in {}",
                query,
                subject,
            );
        }
    }

    #[test]
    fn score_matches_test() {
        let cases: Vec<(Query, Subject, Vec<usize>)> = vec![
            // Exact acronym
            ("fft".into(), "FirstFactoryTest".into(), vec![0, 5, 12]),
            // Extra acronym letters
            ("fft".into(), "FirstFactoryTest.ts".into(), vec![0, 5, 12]),
            // Exact match
            ("core".into(), "0core0app.rb".into(), vec![1, 2, 3, 4]),
            // Exact match, second position is better
            (
                "core".into(),
                "0core0app_core.rb".into(),
                vec![10, 11, 12, 13],
            ),
            // Consecutive letters
            ("core".into(), "controller".into(), vec![0, 1, 4, 8]),
        ];

        for (query, subject, expected) in cases {
            let result = score(&query, &subject);
            assert!(result.is_some());

            let result = result.unwrap();
            assert_eq!(
                result.matches, expected,
                "Expected {} to have matches {:?} inside {:?}",
                query, expected, subject,
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
                result_a.score() > result_b.score(),
                "Expected {} to have a higher score than {} inside {:?}",
                a,
                b,
                subject,
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
            result_a.score() > result_b.score(),
            "Expected {} to have a higher score in {:?} than in {:?}. A {:?} <= B {:?}",
            query,
            a.text,
            b.text,
            result_a.score(),
            result_b.score(),
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
            (
                "modelcore".into(),
                "model-0-core-000.x".into(),
                "model-0-core0-00.x".into(),
            ),
            // start-of-word > end-of-word
            (
                "modelcore".into(),
                "model-0-core0-00.x".into(),
                "model-0core-0000.x".into(),
            ),
            // end-of-word > middle-of-word
            (
                "modelcore".into(),
                "model-0core-0000.x".into(),
                "model-0core0-000.x".into(),
            ),
            // middle-of-word > scattered letters
            (
                "modelcore".into(),
                "model-0core0-000.x".into(),
                "model-controller.x".into(),
            ),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_consecutive_letters_full_word_preference_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // full-word > start-of-word
            (
                "modelcorex".into(),
                "model-0-core-000.x".into(),
                "model-0-core0-00.x".into(),
            ),
            // start-of-word > end-of-word
            (
                "modelcorex".into(),
                "model-0-core0-00.x".into(),
                "model-0core-0000.x".into(),
            ),
            // end-of-word > middle-of-word
            (
                "modelcorex".into(),
                "model-0core-0000.x".into(),
                "model-0core0-000.x".into(),
            ),
            // middle-of-word > scattered letters
            (
                "modelcorex".into(),
                "model-0core0-000.x".into(),
                "model-controller.x".into(),
            ),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_consecutive_letters_preference_test_vs_directory_depth_test() {
        let cases: Vec<(Query, Subject, Subject)> = vec![
            // full-word > start-of-word
            (
                "model core".into(),
                "0/0/0/0/model/core_000.x".into(),
                "0/0/0/model/core0_00.x".into(),
            ),
            // start-of-word > end-of-word
            (
                "model core".into(),
                "0/0/0/model/core0_00.x".into(),
                "0/0/model/0core_00.x".into(),
            ),
            // end-of-word > middle-of-word
            (
                "model core".into(),
                "0/0/model/0core_00.x".into(),
                "0/model/0core0_0.x".into(),
            ),
            // middle-of-word > scattered letters
            (
                "model core".into(),
                "0/model/0core0_0.x".into(),
                "model/controller.x".into(),
            ),
        ];

        for (query, a, b) in cases {
            assert_scores_between_subjects(query, a, b);
        }
    }

    #[test]
    fn score_on_consecutive_letters_score_higher_than_scattered_test() {
        assert_scores_between_subjects(
            "acon".into(),
            "applicatio_controller.rb".into(),
            "application.rb".into(),
        );
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
        assert_scores_between_subjects(
            "core app".into(),
            "controller/core/app.rb".into(),
            "controller/app.rb".into(),
        );
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
