use async_std::sync::Arc;
use std::fmt;
use std::slice::Iter;
use unicode_segmentation::UnicodeSegmentation;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Text: The Arc version of Letters
pub type Text = Arc<Letters>;

#[derive(Debug, Clone)]
pub struct TextBuilder;

impl TextBuilder {
    pub fn build(string: String) -> Text {
        let text: Letters = string.into();

        Arc::new(text)
    }
}

// Letters: The collection of letters (Graphemes) of a string
// This is really private, the idea is to use Text to use Arc
// to prevent extra copies of Strings
#[derive(Debug, Clone)]
pub struct Letters {
    // TODO: Remove pub and implement a custom truncate method
    pub string: String,
    graphemes: Vec<String>,
    graphemes_lw: Vec<String>,
}

impl Letters {
    pub fn new(string: String) -> Self {
        let graphemes = string
            .graphemes(true)
            .map(|s| String::from(s))
            .collect::<Vec<_>>();

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
