//! Individual components configuration

use super::styling::{Rule, Style};
use serde::Deserialize;

const DEFAULT_HEIGHT: usize = 6;
const MIN_HEIGHT: usize = 3;
const MIN_WIDTH: usize = 4;

#[derive(Deserialize, Clone, Debug, PartialEq)]
enum Mode {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "inline")]
    Inline,
}

impl Mode {
    pub fn is_full(&self) -> bool {
        matches!(self, Mode::Full)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Full
    }
}

/// Main screen configuration options
#[derive(Deserialize, Clone, Debug, Default)]
pub struct ScreenConfig {
    #[serde(default)]
    mode: Mode,
    #[serde(default, alias = "columns")]
    width: Option<usize>,
    #[serde(default, alias = "lines")]
    height: Option<usize>,
    #[serde(skip)]
    full_width: usize,
    #[serde(skip)]
    full_height: usize,
}

impl ScreenConfig {
    pub fn inline_mode(&mut self) {
        self.mode = Mode::Inline;
    }

    pub fn full_mode(&mut self) {
        self.mode = Mode::Full;
    }

    pub fn is_full(&self) -> bool {
        self.mode.is_full()
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = Some(height)
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = Some(width)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width(), self.height())
    }

    pub fn width(&self) -> usize {
        let width = match self.mode {
            Mode::Full => self.full_width,
            Mode::Inline => self.width.unwrap_or(self.full_width),
        };

        MIN_WIDTH.max(width)
    }

    pub fn height(&self) -> usize {
        let height = match self.mode {
            Mode::Full => self.full_height,
            Mode::Inline => self.height.unwrap_or(DEFAULT_HEIGHT),
        };

        MIN_HEIGHT.max(height)
    }

    pub fn set_full_size(&mut self, width: usize, height: usize) {
        self.full_width = width;
        self.full_height = height;
    }
}

/// Prompt UI component configuration options
///
/// The prompt is where you write the search query
#[derive(Deserialize, Debug, Clone, Default)]
pub struct PromptConfig {
    symbol: Option<String>,
    style: Option<Style>,
    style_symbol: Option<Style>,
}

impl PromptConfig {
    /// Symbol used before the query
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("> "),
        }
    }

    /// Query styles
    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }

    /// Symbol styles
    pub fn style_symbol(&self) -> Style {
        match &self.style_symbol {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }
}

/// Gauge UI component configuration options
///
/// The gauge indicates the number of matched strings vs total
#[derive(Deserialize, Debug, Clone, Default)]
pub struct GaugeConfig {
    prefix: Option<String>,
    symbol: Option<String>,
    style: Option<Style>,
}

impl GaugeConfig {
    /// Symbol used to separate current vs total numbers
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("/"),
        }
    }

    /// Text used before the numbers
    pub fn prefix(&self) -> String {
        match &self.prefix {
            Some(pref) => pref.clone(),
            None => String::from("  "),
        }
    }

    /// Style for the gauge
    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }
}

/// UI options for each candidate in the list
///
/// A candidate is a string not selected
#[derive(Deserialize, Debug, Clone, Default)]
pub struct CandidateConfig {
    symbol: Option<String>,
    style: Option<Style>,
    style_symbol: Option<Style>,
    style_match: Option<Style>,
}

impl CandidateConfig {
    /// Symbol shown before the candidate's string
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("  "),
        }
    }

    /// Style for the whole string
    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }

    /// Style for the symbol
    pub fn style_symbol(&self) -> Style {
        match &self.style_symbol {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }

    /// Style for the matches inside the candidate
    pub fn style_match(&self) -> Style {
        match &self.style_match {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Underline, Rule::Bold]),
        }
    }
}

/// UI options for the selected candidate in the list
#[derive(Deserialize, Debug, Clone, Default)]
pub struct SelectionConfig {
    symbol: Option<String>,
    style: Option<Style>,
    style_symbol: Option<Style>,
    style_match: Option<Style>,
}

impl SelectionConfig {
    /// Symbol shown before the candidate's string
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("* "),
        }
    }

    /// Style for the whole string
    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Reverse]),
        }
    }

    /// Style for the symbol
    pub fn style_symbol(&self) -> Style {
        match &self.style_symbol {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Reverse]),
        }
    }

    /// Style for the matches inside the candidate
    pub fn style_match(&self) -> Style {
        match &self.style_match {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Underline, Rule::Bold, Rule::Reverse]),
        }
    }
}
