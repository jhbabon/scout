//! Collection of functions that calculate scores based on different heuristics

use super::predicates::*;
use super::types::*;
use crate::common::Text;

const WM: f32 = 150.0;
const POSITION_BOOST: f32 = 100.0;
// The character from 0..POSITION_BONUS receive a greater bonus for being at the start of string.
const POSITION_BONUS: f32 = 20.0;
const POSITION_MIN: f32 = 0.0;
// Full path length at which the whole match score is halved.
const TAU_SIZE: f32 = 150.0;

/// Given a qualified score (quality), calculate how good it is based on query's
/// and subject's length and position
pub fn score_quality(query_len: usize, subject_len: usize, quality: f32, position: f32) -> f32 {
    2.0 * (query_len as f32)
        * ((WM * quality) + score_position(position))
        * score_size(query_len, subject_len)
}

/// Calculate the score associated to a given position
pub fn score_position(position: f32) -> f32 {
    if position < POSITION_BONUS {
        let sc = POSITION_BONUS - position;
        POSITION_BOOST + (sc * sc)
    } else {
        POSITION_MIN.max((POSITION_BOOST + POSITION_BONUS) - position)
    }
}

/// Calculate the score associated to query's and subject's length
pub fn score_size(query_len: usize, subject_len: usize) -> f32 {
    let penalty = (subject_len as isize - query_len as isize).abs();

    TAU_SIZE / (TAU_SIZE + penalty as f32)
}

/// Calculate the score of the acronyms represented by the query, if any
pub fn score_acronyms(query: &Query, subject: &Text) -> Option<AcronymResult> {
    // single char strings are not an acronym
    if query.len() <= 1 || subject.len() <= 1 {
        return None;
    }

    let mut matches = vec![];
    let mut count = 0;
    let mut sep_count = 0;
    let mut sum_position = 0;
    let mut same_case = 0;

    let query_iter = query.lowercase_iter().enumerate();
    let mut subject_iter = subject.lowercase_iter().enumerate();

    let mut progress = 0;
    'query_loop: for (qindex, query_grapheme) in query_iter {
        if progress == subject.len() {
            // The subject text has been consumed, we can stop
            break 'query_loop;
        }

        'subject_loop: for (index, subject_grapheme) in &mut subject_iter {
            progress += 1;

            if query_grapheme == subject_grapheme {
                if is_word_separator(query_grapheme) {
                    // separators don't score points, but we keep track of them
                    sep_count += 1;

                    break 'subject_loop;
                } else if is_start_of_word(subject, index) {
                    // only count graphemes that are start of a word
                    sum_position += index;
                    count += 1;

                    // we don't need to trace back the matches
                    // we only return the acronym score if the number of
                    // acronym's matches equals the query length, with means
                    // the number of matches will equal that length as well
                    matches.push(index);

                    if query.grapheme_at(qindex) == subject.grapheme_at(index) {
                        same_case += 1;
                    }

                    break 'subject_loop;
                }
            }
        }
    }

    if count < 2 {
        return None;
    }

    let full_world = if count == query.len() {
        // the query doesn't have any separator so it might be
        // the unique acronym inside subject
        is_a_unique_acronym(subject, count)
    } else {
        false
    };
    let score = score_pattern(count, query.len(), same_case, true, full_world);

    if score <= 0.0 {
        return None;
    }

    let position = sum_position as f32 / count as f32;

    Some(AcronymResult::new(
        score,
        position,
        count + sep_count,
        matches,
    ))
}

/// Calculate the score of an exact match, if any
pub fn score_exact_match(query: &Query, subject: &Text) -> Option<ExactMatchResult> {
    let (mut position, mut same_case) = sequence_position(query, subject, 0)?;

    let mut is_start;
    is_start = is_start_of_word(subject, position);

    if !is_start {
        // try a second sequence to see if is better (word start) than the previous one
        // we don't want to try more than twice
        if let Some((sec_position, sec_same_case)) =
            sequence_position(query, subject, position + query.len())
        {
            is_start = is_start_of_word(subject, sec_position);

            if is_start {
                position = sec_position;
                same_case = sec_same_case;
            }
        }
    }

    let is_end = is_end_of_word(subject, (position + query.len()) - 1);
    let score = score_quality(
        query.len(),
        subject.len(),
        score_pattern(query.len(), query.len(), same_case, is_start, is_end),
        position as f32,
    );
    let matches: Vec<usize> = (position..(position + query.len())).collect();

    Some(ExactMatchResult::new(score, matches))
}

/// Shared logic to calculate scores in different scenarios:
///   * exact match
///   * acronyms
///   * consecutive matches
///
/// Ensure that the pattern length dominates the score, then refine it
/// to take into account case sensitive matches.
///
/// It also takes into account structural quality of the pattern with word
/// boundaries (start and end).
pub fn score_pattern(
    count: usize,
    len: usize,
    same_case: usize,
    is_start: bool,
    is_end: bool,
) -> f32 {
    let mut sc = count;
    let mut bonus = 6;

    if same_case == count {
        bonus += 2;
    }

    if is_start {
        bonus += 3;
    }

    if is_end {
        bonus += 1;
    }

    if count == len {
        if is_start {
            if same_case == len {
                sc += 2;
            } else {
                sc += 1;
            }
        }

        if is_end {
            bonus += 1;
        }
    }

    (same_case + (sc * (sc + bonus))) as f32
}

/// Forward search for a sequence of consecutive characters and return the score
pub fn score_consecutives(
    query: &Query,
    subject: &Text,
    query_position: usize,
    subject_position: usize,
    is_start: bool,
) -> f32 {
    let query_left = query.len() - query_position;
    let subject_left = subject.len() - subject_position;

    let left = if subject_left < query_left {
        subject_left
    } else {
        query_left
    };

    let mut same_case = 0;
    let mut sz = 1;

    if query.grapheme_at(query_position) == subject.grapheme_at(subject_position) {
        same_case += 1;
    }

    let query_iter = query.lowercase_iter().enumerate().skip(query_position + 1);
    let mut subject_iter = subject
        .lowercase_iter()
        .enumerate()
        .skip(subject_position + 1);

    let mut subject_cursor = subject_position;

    for (qindex, query_grapheme) in query_iter {
        if let Some((index, subject_grapheme)) = subject_iter.next() {
            if query_grapheme == subject_grapheme {
                subject_cursor = index;

                if query.grapheme_at(qindex) == subject.grapheme_at(index) {
                    same_case += 1;
                }
            } else {
                break;
            }
        } else {
            break;
        }

        if sz >= left {
            break;
        }

        sz += 1;
    }

    if sz == 1 {
        let score = 1 + 2 * same_case;

        return score as f32;
    }

    let is_end = is_end_of_word(subject, subject_cursor);

    score_pattern(sz, query.len(), same_case, is_start, is_end)
}

/// Calculate the score of a character based on its position and calculated
/// acronym and consecutive scores around it.
pub fn score_character(
    position: usize,
    is_start: bool,
    acronym_score: f32,
    consecutive_score: f32,
) -> f32 {
    let position_score = score_position(position as f32);

    let mut start_bonus = 0.0;
    let mut score = consecutive_score;
    if is_start {
        start_bonus = 10.0;
        if acronym_score > consecutive_score {
            score = acronym_score;
        }
    }

    position_score + (WM * (score + start_bonus))
}

/// Get the position of the exact sequence of Query contained in Subject, if any
/// It also returns the number of same case graphemes in the sequence
fn sequence_position(query: &Query, subject: &Text, skip: usize) -> Option<(usize, usize)> {
    let mut query_iter = query.lowercase_iter().enumerate();
    let mut subject_iter = subject.lowercase_iter().enumerate().skip(skip);

    let mut sequence = false;
    let mut position = 0;
    let mut same_case = 0;

    while let Some((qindex, query_grapheme)) = query_iter.next() {
        let (index, subject_grapheme) = subject_iter.next()?;

        if query_grapheme == subject_grapheme {
            if !sequence {
                position = index;
            }
            sequence = true;

            if query.grapheme_at(qindex) == subject.grapheme_at(index) {
                same_case += 1
            }
        } else {
            same_case = 0;
            sequence = false;

            // rewind the iterator
            query_iter = query.lowercase_iter().enumerate();
        }
    }

    Some((position, same_case))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::TextBuilder;

    #[test]
    fn score_acronyms_with_no_results_test() {
        let cases = vec![
            ("fft", "FirstFactoryTests"),
            ("ff/t", "FirstFactory/Tests"),
            ("y̆f公🍣", "Y̆/first/公/🍣.js"),
        ];

        for (q, s) in cases {
            let query = Query::from(q);
            let subject = TextBuilder::build(s);
            let result = score_acronyms(&query, &subject);

            assert!(result.is_some(), "Expected {} be inside {}", query, subject);
            assert!(
                result.unwrap().score > 0.0,
                "Expected {} be inside {}",
                query,
                subject
            );
        }
    }

    #[test]
    fn score_acronyms_test() {
        let cases = vec![
            ("f", "fact"),
            ("fft", "f"),
            ("iae", "FirstFactoryTests.html"),
        ];

        for (q, s) in cases {
            let query = Query::from(q);
            let subject = TextBuilder::build(s);
            let result = score_acronyms(&query, &subject);

            assert!(
                result.is_none(),
                "Expected {} not to be inside {}",
                query,
                subject
            );
        }
    }

    #[test]
    fn score_exact_match_with_no_results_test() {
        let cases = vec![
            ("test", "t0e0s0t"),
            ("foo", "f oo"),
            ("y̆公🍣", "y̆x公x🍣.js"),
        ];

        for (q, s) in cases {
            let query = Query::from(q);
            let subject = TextBuilder::build(s);
            let result = score_exact_match(&query, &subject);

            assert!(
                result.is_none(),
                "Expected {} not to be inside {}",
                query,
                subject
            );
        }
    }

    #[test]
    fn score_exact_match_test() {
        let cases = vec![
            ("test", "subject_test.rb"),
            // first is start of word
            ("foo", "foo/foo_test.rb"),
            // second is start of word
            ("foo", "xfoo/foo_test.rb"),
            // none is start of word
            ("foo", "xfooxfoo_test.rb"),
            // different case
            ("foo", "FooTest.rb"),
            ("y̆公🍣", "first/y̆公🍣.js"),
            // different case
            ("y̆公🍣", "First/Y̆公🍣.js"),
        ];

        for (q, s) in cases {
            let query = Query::from(q);
            let subject = TextBuilder::build(s);
            let result = score_exact_match(&query, &subject);

            assert!(
                result.is_some(),
                "Expected {} to score inside {}",
                query,
                subject
            );
            assert!(
                result.unwrap().score > 0.0,
                "Expected {} to score inside {}",
                query,
                subject
            );
        }
    }

    #[test]
    fn score_consecutives_test() {
        let cases = vec![
            // isolated character match
            ("foo", "faa", 0, 0, true),
            // not the whole query is consecutive
            ("foo", "foxo", 0, 0, true),
            ("qfoo", "qabfoxo", 1, 3, false),
            // query finished
            ("foo", "what/foo/bar", 0, 5, true),
            // last subject char is not end of word
            ("foo", "what/foobar", 0, 5, true),
            // firt subject char is not start of word
            ("foo", "whatfoobar", 0, 4, false),
            // subject finished
            ("foo", "what/fo", 0, 5, true),
            ("foo", "fxoox", 1, 2, true),
        ];

        for (q, s, qp, sp, start) in cases {
            let query = Query::from(q);
            let subject = TextBuilder::build(s);

            assert!(
                score_consecutives(&query, &subject, qp, sp, start) > 0.0,
                "Expected {} to score in {}",
                query,
                subject,
            );
        }
    }

    #[test]
    fn sequence_position_test() {
        let cases = vec![
            ("foo", "foo", 0, Some((0, 3))),
            ("foo", "FOO", 0, Some((0, 0))),
            ("Foo", "foo", 0, Some((0, 2))),
            ("foo", "fooxfoo", 0, Some((0, 3))),
            ("foo", "xfoo", 0, Some((1, 3))),
            ("y̆", "xfy̆oo", 0, Some((2, 1))),
            ("y̆", "xfY̆oo", 0, Some((2, 0))),
            ("公", "公", 0, Some((0, 1))),
            ("🍣", "y̆公🍣", 0, Some((2, 1))),
            ("foo", "fooxfoo", 2, Some((4, 3))),
            ("foo", "xfoo", 2, None),
            ("foo", "foxo", 0, None),
            ("foo", "nope", 0, None),
        ];

        for (q, s, skip, expected) in cases {
            let query = Query::from(q);
            let subject = TextBuilder::build(s);

            assert_eq!(
                sequence_position(&query, &subject, skip),
                expected,
                "Expected query {} to be contained in {} at {:?}",
                query,
                subject,
                expected
            );
        }
    }
}
