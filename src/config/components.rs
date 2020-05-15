use super::styling::{Rule, Style};
use serde::Deserialize;

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
        match self {
            Mode::Full => true,
            _ => false,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Full
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ScreenConfig {
    #[serde(default)]
    mode: Mode,
    #[serde(default)]
    style: Option<Style>, // global style
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
        let mut width = self.full_width;

        if let Some(w) = self.width {
            width = w;
        }

        MIN_WIDTH.max(width)
    }

    pub fn height(&self) -> usize {
        let mut height = self.full_height;

        if let Some(w) = self.height {
            height = w;
        }

        MIN_HEIGHT.max(height)
    }

    pub fn set_full_size(&mut self, width: usize, height: usize) {
        self.full_width = width;
        self.full_height = height;
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct PromptConfig {
    symbol: Option<String>,
    style: Option<Style>,
    style_symbol: Option<Style>,
    // TODO: Add separator (?)
}

impl PromptConfig {
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("> "),
        }
    }

    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }

    pub fn style_symbol(&self) -> Style {
        match &self.style_symbol {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct GaugeConfig {
    prefix: Option<String>,
    symbol: Option<String>,
    style: Option<Style>,
}

impl GaugeConfig {
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("/"),
        }
    }

    pub fn prefix(&self) -> String {
        match &self.prefix {
            Some(pref) => pref.clone(),
            None => String::from("  "),
        }
    }

    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CandidateConfig {
    symbol: Option<String>,
    style: Option<Style>,
    style_symbol: Option<Style>,
    style_match: Option<Style>,
}

impl CandidateConfig {
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("  "),
        }
    }

    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }

    pub fn style_symbol(&self) -> Style {
        match &self.style_symbol {
            Some(st) => st.clone(),
            None => Default::default(),
        }
    }

    pub fn style_match(&self) -> Style {
        match &self.style_match {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Underline, Rule::Bold]),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct SelectionConfig {
    symbol: Option<String>,
    style: Option<Style>,
    style_symbol: Option<Style>,
    style_match: Option<Style>,
}

impl SelectionConfig {
    pub fn symbol(&self) -> String {
        match &self.symbol {
            Some(sym) => sym.clone(),
            None => String::from("* "),
        }
    }

    pub fn style(&self) -> Style {
        match &self.style {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Reverse]),
        }
    }

    pub fn style_symbol(&self) -> Style {
        match &self.style_symbol {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Reverse]),
        }
    }

    pub fn style_match(&self) -> Style {
        match &self.style_match {
            Some(st) => st.clone(),
            None => Style::new(vec![Rule::Underline, Rule::Bold]),
        }
    }
}
