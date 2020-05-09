//! Collection of predicate functions over Query and Subject structs

use super::types::*;
use lazy_static;
use std::collections::HashSet;

const ACRONYM_FREQUENCY: usize = 12;

lazy_static! {
    static ref WORD_SEPARATORS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert(" ");
        s.insert(".");
        s.insert("-");
        s.insert("_");
        s.insert("/");
        s.insert("\\");

        s
    };
}

/// Check whether a query is inside a subject or not
pub fn is_match(query: &Query, subject: &Subject) -> bool {
    let mut query_iter = query.graphemes_lw.iter();
    let mut subject_iter = subject.graphemes_lw.iter();

    let mut count = 0;
    let mut done = false;
    while let Some(query_grapheme) = query_iter.next() {
        if done {
            // The subject.graphemes_lw collection is done, but not the query.graphemes_lw
            // which means that the query is not inside the subject
            return false;
        }

        'inner: while let Some(subject_grapheme) = subject_iter.next() {
            count += 1;

            if query_grapheme == subject_grapheme {
                // this grapheme is inside the subject, we can move to the next one
                break 'inner;
            }
        }

        if count == subject.len {
            done = true;
        }
    }

    true
}

/// Check whether the subject has a unique acronym of the given size
pub fn is_a_unique_acronym(subject: &Subject, acronym_size: usize) -> bool {
    let mut count = 0;

    // Assume one acronym every (at most) ACRONYM_FREQUENCY characters
    // on average. This is done to filter long paths
    if subject.len > (ACRONYM_FREQUENCY * acronym_size) {
        return false;
    }

    let mut iter = subject.graphemes.iter().enumerate();

    while let Some((index, _)) = iter.next() {
        if is_start_of_word(subject, index) {
            // only start of word graphemes are considered part of an acronym
            count += 1;

            // If the acronym count is more than acronym_size
            // then we do not have 1:1 relationship
            if count > acronym_size {
                return false;
            }
        }
    }

    true
}

/// Check whether the grapheme at given the position is a start of word or not
///
/// A grapheme is a start of word if it is either:
///   * (a) in the first position of the subject
///   * (b) following a word separator
///   * (c) capital letter after a lowercase letter (camelCase rule)
pub fn is_start_of_word(subject: &Subject, position: usize) -> bool {
    if position == 0 {
        return true; // (a)
    }

    let prev_position = position - 1;

    let current_grapheme = &subject.graphemes[position];
    let prev_grapheme = &subject.graphemes[prev_position];

    is_word_separator(prev_grapheme) // (b)
        || ((current_grapheme != &subject.graphemes_lw[position])
            && (prev_grapheme == &subject.graphemes_lw[prev_position])) // (c)
}

/// Check whether the grapheme at given the position is an end of word or not
///
/// A grapheme is an end of word if it is either:
///   * (a) in the last position of the subject
///   * (b) followed by a word separator
///   * (c) lowercase letter followed by a capital letter (camelCase rule)
pub fn is_end_of_word(subject: &Subject, position: usize) -> bool {
    if position == subject.len - 1 {
        return true; // (a)
    }

    let next_position = position + 1;

    let current_grapheme = &subject.graphemes[position];
    let next_grapheme = &subject.graphemes[next_position];

    is_word_separator(next_grapheme) // (b)
        || ((current_grapheme == &subject.graphemes_lw[position])
            && (next_grapheme != &subject.graphemes_lw[next_position])) // (c)
}

/// Check whether the given grapheme is a word separator
pub fn is_word_separator(grapheme: &str) -> bool {
    WORD_SEPARATORS.contains(grapheme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_match_test() {
        let cases = vec![
            (Query::from("y̆公🍣"), Subject::from("y̆公🍣"), true),
            (Query::from("y̆公🍣"), Subject::from("y̆f公x🍣.rb"), true),
            (Query::from("foo"), Subject::from("foo"), true),
            (Query::from("f oo"), Subject::from("f   oo"), true),
            (Query::from("foo"), Subject::from("fXoXo"), true),
            (Query::from("foo"), Subject::from("f_o.o"), true),
            (Query::from("FoO"), Subject::from("foo"), true),
            (Query::from("foo"), Subject::from("FxOxox"), true),
            (Query::from("foo"), Subject::from("bar"), false),
            (Query::from("foo"), Subject::from("fo"), false),
            (Query::from("f oo"), Subject::from("fo o"), false),
        ];

        for (query, subject, expected) in cases {
            assert_eq!(
                is_match(&query, &subject),
                expected,
                "Expected {:?} to be in {:?}",
                query.string,
                subject.text
            );
        }
    }

    #[test]
    fn is_a_unique_acronym_test() {
        let cases = vec![
            (Subject::from("FactoryFiles"), 2, true),
            (Subject::from("factoryFiles"), 2, true),
            (Subject::from("factory files"), 2, true),
            (Subject::from("FactoryFilesTests"), 2, false),
            (Subject::from("FxxFxxfxxfxxfxxfxxfxxfxxfxxfxx"), 2, false), // filter out long paths
        ];

        for (subject, size, expected) in cases {
            assert_eq!(
                is_a_unique_acronym(&subject, size),
                expected,
                "Expected {:?} to have a unique acronym of size {:?}",
                subject.text,
                size
            );
        }
    }

    #[test]
    fn is_start_of_word_test() {
        let cases = vec![
            (Subject::from("FactoryFiles"), 0, true),
            (Subject::from("FactoryFiles"), 7, true),
            (Subject::from("factory files"), 8, true),
            (Subject::from("fuzzy.rs"), 6, true),

            (Subject::from("FactoryFiles"), 11, false),
            (Subject::from("FactoryFiles"), 3, false),
            (Subject::from("FactoryFiles"), 1, false),
            (Subject::from("fuzzy.rs"), 5, false),
            (Subject::from("FFiles"), 1, false),
            (Subject::from("factory files"), 6, false),
        ];

        for (subject, position, expected) in cases {
            assert_eq!(
                is_start_of_word(&subject, position),
                expected,
                "Expected {:?} to have a start of word at {:?}",
                subject.text,
                position
            );
        }
    }

    #[test]
    fn is_end_of_word_test() {
        let cases = vec![
            (Subject::from("FactoryFiles"), 11, true),
            (Subject::from("FactoryFiles"), 6, true),
            (Subject::from("factory files"), 6, true),
            (Subject::from("fuzzy.rs"), 7, true),
            (Subject::from("fuzzy.rs"), 4, true),

            (Subject::from("FactoryFiles"), 0, false),
            (Subject::from("FactoryFiles"), 1, false),
            (Subject::from("FactoryFiles"), 3, false),
            (Subject::from("fuzzy.rs"), 5, false),
        ];

        for (subject, position, expected) in cases {
            assert_eq!(
                is_end_of_word(&subject, position),
                expected,
                "Expected {:?} to have an end of word at {:?}",
                subject.text,
                position
            );
        }
    }

    #[test]
    fn is_word_separator_test() {
        let cases = vec![
            (" ", true),
            (".", true),
            ("-", true),
            ("_", true),
            ("/", true),
            ("\\", true),
            ("a", false),
            ("y̆", false),
            ("公", false),
            ("🍣", false),
        ];

        for (grapheme, expected) in cases {
            assert_eq!(
                is_word_separator(grapheme),
                expected,
                "Expected {:?} to be {:?}",
                grapheme,
                expected
            );
        }
    }
}
