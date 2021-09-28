//! Collection of shared predicate functions used through scoring functions

use super::types::*;
use crate::common::Text;
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

lazy_static! {
    static ref OPTIONAL_GRAPHEMES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert(" ");
        s.insert(":");
        s.insert("-");
        s.insert("_");
        s.insert("/");
        s.insert("\\");

        s
    };
}

/// Check whether a query is inside a subject or not
pub fn is_match(query: &Query, subject: &Text) -> bool {
    let query_iter = query.lowercase_iter().filter(|g| !is_optional(&g));
    let mut subject_iter = subject.lowercase_iter();

    let mut query_count = 0;
    let mut subject_count = 0;
    let mut matching = 0;
    'query_loop: for query_grapheme in query_iter {
        query_count += 1;

        if subject_count == subject.len() {
            break 'query_loop;
        }

        'subject_loop: for subject_grapheme in &mut subject_iter {
            subject_count += 1;

            if query_grapheme == subject_grapheme {
                matching += 1;
                // this grapheme is inside the subject, we can move to the next one
                break 'subject_loop;
            }
        }
    }

    matching == query_count
}

/// Check whether the subject has a unique acronym of the given size
pub fn is_a_unique_acronym(subject: &Text, acronym_size: usize) -> bool {
    let mut count = 0;
    let len = subject.len();

    // Assume one acronym every (at most) ACRONYM_FREQUENCY characters
    // on average. This is done to filter long paths
    if len > (ACRONYM_FREQUENCY * acronym_size) {
        return false;
    }

    let mut index = 0;

    while index < len {
        if is_start_of_word(subject, index) {
            // only start of word graphemes are considered part of an acronym
            count += 1;

            // If the acronym count is more than acronym_size
            // then we do not have 1:1 relationship
            if count > acronym_size {
                return false;
            }
        }

        index += 1;
    }

    true
}

/// Check whether the grapheme at given the position is a start of word or not
///
/// A grapheme is a start of word if it is either:
///   * (a) in the first position of the subject
///   * (b) following a word separator
///   * (c) capital letter after a lowercase letter (camelCase rule)
pub fn is_start_of_word(subject: &Text, position: usize) -> bool {
    if position == 0 {
        return true; // (a)
    }

    let prev_position = position - 1;

    let current_grapheme = subject.grapheme_at(position);
    let prev_grapheme = subject.grapheme_at(prev_position);

    is_word_separator(prev_grapheme) // (b)
        || ((current_grapheme != subject.lowercase_grapheme_at(position))
            && (prev_grapheme == subject.lowercase_grapheme_at(prev_position))) // (c)
}

/// Check whether the grapheme at given the position is an end of word or not
///
/// A grapheme is an end of word if it is either:
///   * (a) in the last position of the subject
///   * (b) followed by a word separator
///   * (c) lowercase letter followed by a capital letter (camelCase rule)
pub fn is_end_of_word(subject: &Text, position: usize) -> bool {
    if position == subject.last_index() {
        return true; // (a)
    }

    let next_position = position + 1;

    let current_grapheme = subject.grapheme_at(position);
    let next_grapheme = subject.grapheme_at(next_position);

    is_word_separator(next_grapheme) // (b)
        || ((current_grapheme == subject.lowercase_grapheme_at(position))
            && (next_grapheme != subject.lowercase_grapheme_at(next_position))) // (c)
}

/// Check whether the given grapheme is a word separator
pub fn is_word_separator(grapheme: &str) -> bool {
    WORD_SEPARATORS.contains(grapheme)
}

fn is_optional(grapheme: &str) -> bool {
    OPTIONAL_GRAPHEMES.contains(grapheme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::TextBuilder;

    #[test]
    fn is_match_test() {
        let cases = vec![
            (Query::from("y̆公🍣"), TextBuilder::build("y̆公🍣"), true),
            (Query::from("y̆公🍣"), TextBuilder::build("y̆f公x🍣.rb"), true),
            (Query::from("foo"), TextBuilder::build("foo"), true),
            (Query::from("f oo"), TextBuilder::build("f   oo"), true),
            (Query::from("foo"), TextBuilder::build("fXoXo"), true),
            (Query::from("foo"), TextBuilder::build("f_o.o"), true),
            (Query::from("FoO"), TextBuilder::build("foo"), true),
            (Query::from("foo"), TextBuilder::build("FxOxox"), true),
            (Query::from("foo"), TextBuilder::build("bar"), false),
            (Query::from("foo"), TextBuilder::build("fo"), false),
            (Query::from("f oo"), TextBuilder::build("fo o"), true),
            (
                Query::from("ffb"),
                TextBuilder::build("activerecord/test/fixtures/faces.yml"),
                false,
            ),
        ];

        for (query, subject, expected) in cases {
            assert_eq!(
                is_match(&query, &subject),
                expected,
                "Query {}. Subject {}",
                query,
                subject
            );
        }
    }

    #[test]
    fn is_a_unique_acronym_test() {
        let cases = vec![
            (TextBuilder::build("FactoryFiles"), 2, true),
            (TextBuilder::build("factoryFiles"), 2, true),
            (TextBuilder::build("factory files"), 2, true),
            (TextBuilder::build("FactoryFilesTests"), 2, false),
            (
                TextBuilder::build("FxxFxxfxxfxxfxxfxxfxxfxxfxxfxx"),
                2,
                false,
            ), // filter out long paths
        ];

        for (subject, size, expected) in cases {
            assert_eq!(
                is_a_unique_acronym(&subject, size),
                expected,
                "Expected {} to have a unique acronym of size {:?}",
                subject,
                size
            );
        }
    }

    #[test]
    fn is_start_of_word_test() {
        let cases = vec![
            (TextBuilder::build("FactoryFiles"), 0, true),
            (TextBuilder::build("FactoryFiles"), 7, true),
            (TextBuilder::build("factory files"), 8, true),
            (TextBuilder::build("fuzzy.rs"), 6, true),
            (TextBuilder::build("FactoryFiles"), 11, false),
            (TextBuilder::build("FactoryFiles"), 3, false),
            (TextBuilder::build("FactoryFiles"), 1, false),
            (TextBuilder::build("fuzzy.rs"), 5, false),
            (TextBuilder::build("FFiles"), 1, false),
            (TextBuilder::build("factory files"), 6, false),
        ];

        for (subject, position, expected) in cases {
            assert_eq!(
                is_start_of_word(&subject, position),
                expected,
                "Expected {} to have a start of word at {:?}",
                subject,
                position
            );
        }
    }

    #[test]
    fn is_end_of_word_test() {
        let cases = vec![
            (TextBuilder::build("FactoryFiles"), 11, true),
            (TextBuilder::build("FactoryFiles"), 6, true),
            (TextBuilder::build("factory files"), 6, true),
            (TextBuilder::build("fuzzy.rs"), 7, true),
            (TextBuilder::build("fuzzy.rs"), 4, true),
            (TextBuilder::build("FactoryFiles"), 0, false),
            (TextBuilder::build("FactoryFiles"), 1, false),
            (TextBuilder::build("FactoryFiles"), 3, false),
            (TextBuilder::build("fuzzy.rs"), 5, false),
        ];

        for (subject, position, expected) in cases {
            assert_eq!(
                is_end_of_word(&subject, position),
                expected,
                "Expected {} to have an end of word at {:?}",
                subject,
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
