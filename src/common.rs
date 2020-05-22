//! Set of common types used through the app

use async_std::sync::Arc;
use std::fmt;
use std::slice::Iter;
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// The Prompt represents the current query, the cursor position in that query and when it was
/// updated.
///
/// When the query in the prompt changes the timestamp is updated to reflect that is a fresh query.
/// This is then used to print to the UI only latest changes.
#[derive(Debug, Clone)]
pub struct Prompt {
    query: Vec<char>,
    cursor: usize,
    timestamp: Instant,
}

impl Prompt {
    pub fn add(&mut self, ch: char) {
        self.query.insert(self.cursor, ch);
        self.cursor += 1;
        self.refresh();
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.query.remove(self.cursor);
            self.refresh();

            return true;
        }

        false
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.refresh();
    }

    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.cursor < self.len() {
            self.cursor += 1;
        }
    }

    pub fn cursor_at_end(&mut self) {
        self.cursor = self.len();
    }

    pub fn cursor_at_start(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_until_end(&self) -> usize {
        if self.len() < self.cursor {
            0
        } else {
            self.len() - self.cursor
        }
    }

    pub fn as_string(&self) -> String {
        self.query.iter().collect()
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn len(&self) -> usize {
        self.query.len()
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    pub fn refresh(&mut self) {
        self.timestamp = Instant::now();
    }
}

impl From<&String> for Prompt {
    fn from(string: &String) -> Self {
        let query = string.chars().collect::<Vec<char>>();
        let cursor = query.len();

        Self {
            query,
            cursor,
            ..Default::default()
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            cursor: 0,
            query: vec![],
        }
    }
}

/// The Arc version of Letters
pub type Text = Arc<Letters>;

/// Text type builder
#[derive(Debug, Clone)]
pub struct TextBuilder;

impl TextBuilder {
    pub fn build(string: &str) -> Text {
        let text: Letters = string.into();

        Arc::new(text)
    }
}

/// The collection of letters (Graphemes) of a string.
///
/// These letters are the core part of the fuzzy matching algorithm.
///
/// This type is not used directly but through the Text type,
/// which is an Arc wrapper around this type. We use Arc to reduce
/// the String allocations between tasks as much as possible.
#[derive(Debug, Clone)]
pub struct Letters {
    string: String,
    graphemes: Vec<String>,
    graphemes_lw: Vec<String>,
}

impl Letters {
    pub fn new(string: String) -> Self {
        let graphemes = string.graphemes(true).map(String::from).collect::<Vec<_>>();

        let graphemes_lw = graphemes
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>();

        Self {
            string,
            graphemes,
            graphemes_lw,
        }
    }

    pub fn len(&self) -> usize {
        self.graphemes.len()
    }

    pub fn last_index(&self) -> usize {
        let len = self.len();

        if len == 0 {
            0
        } else {
            len - 1
        }
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
}

impl From<&str> for Letters {
    fn from(string: &str) -> Self {
        Self::new(String::from(string))
    }
}

impl From<String> for Letters {
    fn from(string: String) -> Self {
        Self::new(string)
    }
}

impl fmt::Display for Letters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}
