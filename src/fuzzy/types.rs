use crate::common::{Text, TextBuilder};
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Query {
    text: Text,
    set: HashSet<String>,
}

impl Query {
    pub fn new(text: &Text) -> Self {
        let text = text.clone();
        let set = text.lowercase_iter().map(|s| s.clone()).collect();

        Self { text, set }
    }

    pub fn contains(&self, grapheme: &str) -> bool {
        self.set.contains(grapheme)
    }
}

impl Deref for Query {
    type Target = Text;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl From<&str> for Query {
    fn from(string: &str) -> Self {
        let text = TextBuilder::build(string);

        Self::new(&text)
    }
}

impl From<String> for Query {
    fn from(string: String) -> Self {
        let text = TextBuilder::build(&string);

        Self::new(&text)
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub text: Text,
    score: f32,
    pub matches: Vec<usize>,
}

impl Candidate {
    pub fn new(text: &Text, score: f32, matches: Vec<usize>) -> Self {
        let text = text.clone();

        Self {
            text,
            score,
            matches,
        }
    }

    pub fn score(&self) -> f32 {
        self.score
    }
}

impl Deref for Candidate {
    type Target = Text;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl From<&Text> for Candidate {
    fn from(txt: &Text) -> Self {
        Self::new(txt, 0.0, Vec::new())
    }
}

impl fmt::Display for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Candidate) -> Ordering {
        match self.partial_cmp(other) {
            Some(ord) => ord,
            // let's just assume that if two subject's can't be compared
            // they are equal
            None => Ordering::Equal,
        }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Candidate) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Eq for Candidate {}

impl PartialEq for Candidate {
    fn eq(&self, other: &Candidate) -> bool {
        self.score == other.score
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
