use crate::common::Text;
use async_std::sync::Arc;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::iter::Iterator;
use std::slice::Iter;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Query {
    string: String,
    graphemes: Vec<String>,
    graphemes_lw: Vec<String>,
    graphemes_set: HashSet<String>,
}

impl Query {
    pub fn new(string: String) -> Self {
        let graphemes = string
            .graphemes(true)
            .map(|s| String::from(s))
            .collect::<Vec<_>>();

        let mut graphemes_set = HashSet::new();
        let graphemes_lw = graphemes
            .iter()
            .scan(&mut graphemes_set, |set, s| {
                let lower = s.to_lowercase();
                set.insert(lower.clone());

                Some(lower)
            })
            .collect::<Vec<_>>();

        Self {
            string,
            graphemes,
            graphemes_lw,
            graphemes_set,
        }
    }

    pub fn len(&self) -> usize {
        self.graphemes.len()
    }

    pub fn last_index(&self) -> usize {
        let len = self.len();

        if len == 0 {
            return 0;
        }

        len - 1
    }

    pub fn grapheme_at(&self, index: usize) -> &'_ str {
        &self.graphemes[index]
    }

    pub fn lowercase_grapheme_at(&self, index: usize) -> &'_ str {
        &self.graphemes_lw[index]
    }

    pub fn iter(&self) -> Iter<'_, String> {
        self.graphemes.iter()
    }

    pub fn lowercase_iter(&self) -> Iter<'_, String> {
        self.graphemes_lw.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    pub fn contains(&self, grapheme: &str) -> bool {
        self.graphemes_set.contains(grapheme)
    }
}

impl From<&str> for Query {
    fn from(string: &str) -> Self {
        Self::new(String::from(string))
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

// Candidate replacement. This represent a possible choice
#[derive(Debug, Clone)]
pub struct Subject {
    pub text: Text,
    graphemes: Arc<Vec<String>>,
    graphemes_lw: Arc<Vec<String>>,
    score: f32,
    pub matches: Vec<usize>,
}

impl Subject {
    pub fn new(string: String) -> Self {
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
                .collect::<Vec<_>>(),
        );

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

    pub fn refine(&self, score: f32, matches: Vec<usize>) -> Self {
        let mut refined: Self = self.into();
        refined.score = score;
        refined.matches = matches;

        refined
    }

    pub fn grapheme_at(&self, index: usize) -> &'_ str {
        &self.graphemes[index]
    }

    pub fn lowercase_grapheme_at(&self, index: usize) -> &'_ str {
        &self.graphemes_lw[index]
    }

    pub fn len(&self) -> usize {
        self.graphemes.len()
    }

    pub fn last_index(&self) -> usize {
        let len = self.len();

        if len == 0 {
            return 0;
        }

        len - 1
    }

    pub fn iter(&self) -> Iter<'_, String> {
        self.graphemes.iter()
    }

    pub fn lowercase_iter(&self) -> Iter<'_, String> {
        self.graphemes_lw.iter()
    }

    pub fn score(&self) -> f32 {
        self.score
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

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Ord for Subject {
    fn cmp(&self, other: &Subject) -> Ordering {
        match self.partial_cmp(other) {
            Some(ord) => ord,
            // let's just assume that if two subject's can't be compared
            // they are equal
            None => Ordering::Equal,
        }
    }
}

impl PartialOrd for Subject {
    fn partial_cmp(&self, other: &Subject) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Eq for Subject {}

impl PartialEq for Subject {
    fn eq(&self, other: &Subject) -> bool {
        self.score == other.score
    }
}

impl From<&str> for Subject {
    fn from(string: &str) -> Self {
        Self::new(String::from(string))
    }
}

#[derive(Debug, Clone)]
pub struct AcronymResult {
    pub score: f32,
    pub position: f32,
    pub count: usize,
    pub matches: Vec<usize>,
}

impl AcronymResult {
    pub fn new(score: f32, position: f32, count: usize, matches: Vec<usize>) -> Self {
        Self {
            score,
            position,
            count,
            matches,
        }
    }

    pub fn empty() -> Self {
        // I have no idea why position here is 0.1, to be honest
        // The original code is like this
        //
        // ```js
        // const emptyAcronymResult = new AcronymResult(/*score*/ 0, /*position*/ 0.1, /*count*/ 0);
        // ```
        Self::new(0.0, 0.1, 0, vec![])
    }
}

#[derive(Debug, Clone)]
pub struct ExactMatchResult {
    pub score: f32,
    pub matches: Vec<usize>,
}

impl ExactMatchResult {
    pub fn new(score: f32, matches: Vec<usize>) -> Self {
        Self { score, matches }
    }
}

#[derive(Debug, Clone)]
enum Movement {
    Up,
    Left,
    Diagonal,
    Stop,
}

#[derive(Debug, Clone)]
pub struct TraceMatrix {
    columns: usize,
    matrix: Vec<Movement>,
}

impl TraceMatrix {
    pub fn new(rows: usize, columns: usize) -> Self {
        let matrix = vec![Movement::Up; rows * columns];

        Self { columns, matrix }
    }

    pub fn traceback(self, x: usize, y: usize) -> Vec<usize> {
        let mut row = y;
        let mut column = x;
        let mut position = row * self.columns + column;
        let mut matches: VecDeque<usize> = VecDeque::new();

        loop {
            match self.matrix[position] {
                Movement::Up => {
                    if row == 0 {
                        break;
                    }
                    row -= 1;
                    position -= self.columns;
                }
                Movement::Left => {
                    if column == 0 {
                        break;
                    }
                    column -= 1;
                    position -= 1;
                }
                Movement::Diagonal => {
                    matches.push_front(row);

                    if row == 0 || column == 0 {
                        break;
                    }
                    row -= 1;
                    column -= 1;
                    position -= self.columns + 1;
                }
                Movement::Stop => break,
            };
        }

        matches.into()
    }

    pub fn up_at(&mut self, x: usize, y: usize) {
        self.set(x, y, Movement::Up);
    }

    pub fn left_at(&mut self, x: usize, y: usize) {
        self.set(x, y, Movement::Left);
    }

    pub fn diagonal_at(&mut self, x: usize, y: usize) {
        self.set(x, y, Movement::Diagonal);
    }

    pub fn stop_at(&mut self, x: usize, y: usize) {
        self.set(x, y, Movement::Stop);
    }

    fn set(&mut self, x: usize, y: usize, mv: Movement) {
        let position = y * self.columns + x;
        self.matrix[position] = mv;
    }
}
