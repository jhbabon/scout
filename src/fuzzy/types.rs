use crate::common::Text;
use async_std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

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
pub struct AcronymResult {
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
