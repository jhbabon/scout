// I don't feel like I can build a good fuzzy search algorithm
// so let's use a library, at least for the moment
use log::debug;
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
const WM: f32 = 150.0;
const POSITION_BOOST: f32 = 100.0;
const POSITION_BONUS: f32 = 20.0; // The character from 0..pos_bonus receive a greater bonus for being at the start of string.
const TAU_SIZE: f32 = 150.0; // Full path length at which the whole match score is halved.
const MISS_COEFF: f32 = 0.75; //Max number missed consecutive hit = ceil(miss_coeff*query.length) + 5
const ZERO: f32 = 0.0;

#[derive(Debug, Clone)]
pub struct Query {
    pub string: String,
    pub string_lw: String,
    // I think I can use chars instead of graphemes for filtering (?)
    // but with graphemes I'll have options for UI truncation, etc
    pub graphemes: Vec<String>,
    pub graphemes_lw: Vec<String>,
    pub len: usize,
}

impl Query {
    pub fn new(string: String) -> Self {
        let string_lw = string.to_lowercase();
        let graphemes = string
            .graphemes(true)
            .map(|s| String::from(s))
            .collect::<Vec<_>>();
        let graphemes_lw = graphemes
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>();

        let len = graphemes.len();

        Self {
            string,
            string_lw,
            graphemes,
            graphemes_lw,
            len,
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
    pub text_lw: Text,
    pub graphemes: Arc<Vec<String>>,
    pub graphemes_lw: Arc<Vec<String>>,
    pub score: f32,
    pub matches: Vec<usize>,
    pub len: usize,
}

impl Subject {
    pub fn new(string: String) -> Self {
        let text_lw: Text = string.to_lowercase().into();
        let text: Text = string.into();
        let graphemes = Arc::new(
            text.graphemes(true)
                .map(|s| String::from(s))
                .collect::<Vec<_>>(),
        );
        let graphemes_lw = Arc::new(
            graphemes
                .iter()
                .map(|s| s.to_lowercase())
                .collect::<Vec<_>>()
        );

        let len = graphemes.len();

        let score = 0.0;
        let matches = Vec::new();

        Self {
            text,
            text_lw,
            graphemes,
            graphemes_lw,
            len,
            score,
            matches,
        }
    }
}

impl From<&Subject> for Subject {
    fn from(other: &Subject) -> Self {
        let text = other.text.clone();
        let text_lw = other.text_lw.clone();
        let graphemes = other.graphemes.clone();
        let graphemes_lw = other.graphemes_lw.clone();
        let len = graphemes.len();
        let score = 0.0;
        let matches = Vec::new();

        Self {
            text,
            text_lw,
            graphemes,
            graphemes_lw,
            len,
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

#[derive(Debug, Clone)]
struct AcronymResult {
    pub score: f32,
    pub position: f32,
    pub count: usize,
}

impl AcronymResult {
    pub fn new(score: f32, position: f32, count: usize) -> Self {
        Self {
            score,
            position,
            count,
        }
    }

    pub fn empty() -> Self {
        // I have no idea why position here is 0.1, to be honest
        // The original code is like this
        //
        // ```js
        // const emptyAcronymResult = new AcronymResult(/*score*/ 0, /*position*/ 0.1, /*count*/ 0);
        // ```
        Self::new(0.0, 0.1, 0)
    }
}

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
        let score = score_exact(query.len, subject.len, acronym.score, acronym.position);

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
    // TODO: Individual characters
    // (Smith Waterman algorithm)

    let mut new_subject = Subject::from(subject);
    new_subject.score = acronym.score;

    Some(new_subject)
}

fn is_match(query: &Query, subject: &Subject) -> bool {
    let mut query_iter = query.graphemes_lw.iter();
    let mut subject_iter = subject.graphemes_lw.iter();

    let mut count = 0;
    let mut done = false;
    while let Some(query_grapheme) = query_iter.next() {
        if done {
            // The subject_graphemes collection is done, but not the query_graphemes
            // which means that the query is not inside the subject text
            return false;
        }

        'inner: while let Some(subject_grapheme) = subject_iter.next() {
            count += 1;
            if query_grapheme == subject_grapheme {
                break 'inner;
            }
        }

        if count == subject.len {
            done = true;
        }
    }

    true
}

fn score_acronyms(query: &Query, subject: &Subject) -> AcronymResult {
    // single char strings are not an acronym
    if query.len <= 1 || subject.len <= 1 {
        return AcronymResult::empty();
    }

    let mut count = 0;
    let mut sep_count = 0;
    let mut sum_position = 0;
    let mut same_case = 0;

    let mut query_iter = query.graphemes_lw.iter().enumerate();
    let mut subject_iter = subject.graphemes_lw.iter().enumerate();

    let mut progress = 0;
    let mut done = false;
    'outer: while let Some((qindex, query_grapheme)) = query_iter.next() {
        if done {
            // All of subject has been consumed, we can stop
            break 'outer;
        }

        'inner: while let Some((index, subject_grapheme)) = subject_iter.next() {
            progress += 1;

            if query_grapheme == subject_grapheme {
                if is_separator(query_grapheme) {
                    // separators don't score points, but we keep track of them
                    sep_count += 1;
                    break 'inner;
                } else if is_word_start(index, subject) {
                    // only count chars that are start of a word
                    sum_position += index;
                    count += 1;

                    if &query.graphemes[qindex] == &subject.graphemes[index] {
                        same_case += 1;
                    }

                    break 'inner;
                }
            }
        }

        if progress == subject.len {
            done = true;
        }
    }

    if count < 2 {
        return AcronymResult::empty();
    }

    let mut full_world = false;
    if count == query.len {
        full_world = is_acronym_full_word(query, subject, count);
    }
    let score = score_pattern(count, query.len, same_case, true, full_world);
    let position = sum_position as f32 / count as f32;

    AcronymResult::new(
        score,
        position,
        count + sep_count,
    )
}

fn score_exact_match(query: &Query, subject: &Subject) -> Option<f32> {
    let (mut position, mut same_case) = sequence_position(query, subject, 0)?;

    let mut start;
    start = is_word_start(position, subject);

    if !start {
        // try a second sequence to see if is better (word start) than the previous one
        // we don't want to try more than twice
        if let Some((sec_position, sec_same_case)) = sequence_position(query, subject, position + query.len) {
            start = is_word_start(sec_position, subject);

            if start {
                position = sec_position;
                same_case = sec_same_case;
            }
        }
    }

    let end = is_word_end((position + query.len) - 1, subject);

    let score = score_exact(query.len, subject.len, score_pattern(query.len, query.len, same_case, start, end), position as f32);

    Some(score)
}

fn sequence_position(query: &Query, subject: &Subject, skip: usize) -> Option<(usize, usize)> {
    let mut query_iter = query.graphemes_lw.iter().enumerate();
    let mut subject_iter = subject.graphemes_lw.iter().enumerate().skip(skip);

    let mut sequence = false;
    let mut position = 0;
    let mut same_case = 0;

    while let Some((qindex, query_grapheme)) = query_iter.next() {
        if let Some((index, subject_grapheme)) = subject_iter.next() {
            if query_grapheme == subject_grapheme {
                if !sequence {
                    position = index;
                }
                sequence = true;

                if &query.graphemes[qindex] == &subject.graphemes[index] {
                    same_case += 1
                }
            } else {
                same_case = 0;
                sequence = false;
                // rewind the iterator
                query_iter = query.graphemes_lw.iter().enumerate();
            }
        } else {
            // we finished with the subject
            return None;
        }
    }

    Some((position, same_case))
}

// Shared scoring logic between exact match, consecutive & acronym
// Ensure pattern length dominate the score then refine to take into account case-sensitivity
// and structural quality of the pattern on the overall string (word boundary)
fn score_pattern(count: usize, len: usize, same_case: usize, start: bool, end: bool) -> f32 {
    let mut size = count;
    let mut bonus = 6;

    if same_case == count {
        bonus += 2;
    }

    if start {
        bonus += 3;
    }

    if end {
        bonus += 1;
    }

    if count == len {
        if start {
            if same_case == len {
                size += 2;
            } else {
                size += 1;
            }
        }

        if end {
            bonus += 1;
        }
    }

    (same_case + (size * (size + bonus))) as f32
}

fn score_exact(n: usize, m: usize, quality: f32, position: f32) -> f32 {
    2.0 * (n as f32) * ( (WM * quality) + score_position(position) ) * score_size(n, m)
}

fn score_position(position: f32) -> f32 {
    if position < POSITION_BONUS {
        let sc = POSITION_BONUS - position;
        POSITION_BOOST + (sc * sc)
    } else {
        ZERO.max((POSITION_BOOST + POSITION_BONUS) - position)
    }
}

fn score_size(n: usize, m: usize) -> f32 {
    let calc;
    if m >= n {
        calc = m - n;
    } else {
        calc = n - m;
    }

    TAU_SIZE / (TAU_SIZE + calc as f32)
}

fn is_word_start(position: usize, subject: &Subject) -> bool {
    if position == 0 {
        // first grapheme of subject
        return true;
    }

    let prev_position = position - 1;

    let current_grapheme = &subject.graphemes[position];
    let prev_grapheme = &subject.graphemes[prev_position];

    is_separator(prev_grapheme) // match follows a separator
        || ((current_grapheme != &subject.graphemes_lw[position])
            && (prev_grapheme == &subject.graphemes_lw[prev_position])) // match is Capital in camelCase (preceded by lowercase)
}

fn is_word_end(position: usize, subject: &Subject) -> bool {
    if position == subject.len - 1 {
        // last grapheme of subject
        return true;
    }

    let next_position = position + 1;

    let current_grapheme = &subject.graphemes[position];
    let next_grapheme = &subject.graphemes[next_position];

    is_separator(next_grapheme) // match is followed by a separator
        || ((current_grapheme == &subject.graphemes_lw[position])
            && (next_grapheme != &subject.graphemes_lw[next_position])) // match is lowercase, followed by uppercase
}

fn is_separator(grapheme: &str) -> bool {
    // TODO: Use  HashSet with lazy_static!
    grapheme == " "
        || grapheme == "."
        || grapheme == "-"
        || grapheme == "_"
        || grapheme == "/"
        || grapheme == "\\"
}

// All acronym of candidate must be matched to a character of query to be considered a full word
// acronym
fn is_acronym_full_word(query: &Query, subject: &Subject, nb_acronym_in_query: usize) -> bool {
    let mut count = 0;

    // Heuristic:
    // Assume one acronym every (at most) 12 character on average
    // This filter out long paths, but then they can match on the filename.
    if subject.len > (12 * query.len) {
        return false;
    }

    let mut iter = subject.graphemes.iter().enumerate();

    while let Some((index, _)) = iter.next() {
        // For each char of subject
        // Test if we have an acronym, if so increase acronym count.
        if is_word_start(index, subject) {
            count += 1;

            // If the acronym count is more than nbAcronymInQuery (number of non separator char in query)
            // Then we do not have 1:1 relationship.
            if count > nb_acronym_in_query {
                return false;
            }
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
        let candidates = vec!["Bar", "Fboaor"];
        for candidate in candidates {
            let subject = Subject::from(candidate);

            let result = score(&query, &subject);

            assert!(result.is_some());
        }
    }

    #[test]
    fn acronym_score_test() {
        let query = Query::from("fft");
        let subject = Subject::from("FirstFactoryTests.html");

        let result = score_acronyms(&query, &subject);

        assert!(result.score > 0.0, "{:?}", result)
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
    fn score_exact_match_returns_none_when_the_query_is_not_inside_subject() {
        let query = Query::from("bar");
        let subject = Subject::from("fooxfoo");

        let result = score_exact_match(&query, &subject);

        match result {
            Some(score) => panic!("Found score {:?}", score),
            None => assert!(true),
        }
    }

    #[test]
    fn score_exact_match_returns_none_when_the_query_is_not_exact() {
        let query = Query::from("foo");
        let subject = Subject::from("fobaro");

        let result = score_exact_match(&query, &subject);

        match result {
            Some(score) => panic!("Found score {:?}", score),
            None => assert!(true),
        }
    }

    #[test]
    fn score_exact_match_returns_score_when_the_query_is_inside_subject() {
        let query = Query::from("foo");
        let subject = Subject::from("fooxfoo");

        let result = score_exact_match(&query, &subject);

        match result {
            Some(score) => assert!(score > 0.0, "{:?}", subject),
            None => panic!("No score found"),
        }
    }

    #[test]
    fn score_exact_match_returns_better_score_when_the_query_has_start_of_word() {
        let query = Query::from("foo");
        let subject_a = Subject::from("xfoo Foo");
        let subject_b = Subject::from("xfooafoo");

        let result_a = score_exact_match(&query, &subject_a);
        let result_b = score_exact_match(&query, &subject_b);

        assert!(result_a.is_some(), "No result for {:?}", subject_a);
        assert!(result_b.is_some(), "No result for {:?}", subject_b);

        let score_a = result_a.unwrap();
        let score_b = result_b.unwrap();
        assert!(
            score_a > score_b,
            "Expected score {:?} from {:?} to be greater than {:?} from {:?}",
            score_a,
            subject_a.text,
            score_b,
            subject_b.text
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
