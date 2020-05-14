pub mod styling;

use async_std::fs;
use async_std::os::unix::io::AsRawFd;
use clap::{value_t, ArgMatches};
use serde::Deserialize;
use toml;
// TODO: Better crate imports
use crate::terminal_size::terminal_size;

#[derive(Debug, Deserialize, Clone)]
pub enum Decoration {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "underline")]
    Underline,
    #[serde(rename = "strikethrough")]
    Strikethrough,
    #[serde(rename = "reverse")]
    Reverse,
}

impl Default for Decoration {
    fn default() -> Self {
        Decoration::None
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum FontStyle {
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "bold")]
    Bold,
    #[serde(rename = "italic")]
    Italic,
    #[serde(rename = "dimmed")]
    Dimmed,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Regular
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct StyleConfig {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub decorations: Vec<Decoration>,
    #[serde(default)]
    pub font_style: FontStyle,
    pub color: Option<String>,
    pub background_color: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Styles {
    pub prompt: StyleConfig,
    pub query: StyleConfig,
    pub counter: StyleConfig,
    pub item: StyleConfig,
    pub item_match: StyleConfig,
    pub item_bullet: StyleConfig,
    pub selection: StyleConfig,
    pub selection_match: StyleConfig,
    pub selection_bullet: StyleConfig,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            prompt: StyleConfig {
                text: ">".into(),
                ..Default::default()
            },
            query: Default::default(),
            counter: Default::default(),
            item: Default::default(),
            item_match: StyleConfig {
                font_style: FontStyle::Bold,
                ..Default::default()
            },
            item_bullet: StyleConfig {
                text: " ".into(),
                ..Default::default()
            },
            selection: StyleConfig {
                decorations: vec![Decoration::Reverse],
                ..Default::default()
            },
            selection_match: StyleConfig {
                decorations: vec![Decoration::Reverse],
                font_style: FontStyle::Bold,
                ..Default::default()
            },
            selection_bullet: StyleConfig {
                text: "*".into(),
                decorations: vec![Decoration::Reverse],
                ..Default::default()
            },
        }
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ScreenConfig {
    pub size: (usize, usize),
    // TODO: Enum?
    pub full: bool,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub screen: ScreenConfig,
    pub initial_query: Option<String>,
    #[serde(default)]
    pub styles: Styles,
}

#[derive(Debug)]
pub struct Configurator {
    config: Config,
}

impl Configurator {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn from_str<'a>(&'a mut self, contents: &str) -> &'a mut Self {
        self.config = toml::from_str(contents).unwrap();
        self
    }

    // Set screen size
    pub fn from_ptty<'a>(&'a mut self, ptty: &fs::File) -> &'a mut Self {
        let (rows, cols) = terminal_size(ptty.as_raw_fd()).expect("Error getting terminal size");
        let size = (rows as usize, cols as usize);
        self.config.screen = ScreenConfig { size, full: false };

        self
    }

    pub fn from_args<'a>(&'a mut self, args: &ArgMatches) -> &'a mut Self {
        if args.is_present("inline") {
            self.config.screen.full = false;
            let (width, height) = self.config.screen.size;
            let given = value_t!(args, "lines", usize).unwrap_or(6);
            self.config.screen.size = (width, height.min(given));
        } else {
            self.config.screen.full = true;
        }

        if let Some(q) = args.value_of("search") {
            let q = q.to_string();
            self.config.initial_query = Some(q);
        }

        self
    }

    pub fn build(&self) -> Config {
        // FIXME; Find a better way of passing config around
        //   without cloning all the time
        self.config.clone()
    }
}
