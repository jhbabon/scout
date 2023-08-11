//! Create and define the main configuration through toml files and command line args

pub mod components;
pub mod styling;

use components::*;

use crate::common::Result;
use async_std::fs;
use async_std::os::unix::io::AsRawFd;
use async_std::sync::Arc;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

use crate::terminal_size::terminal_size;

#[derive(Debug, Default)]
pub struct Args {
    // flags
    pub full_screen: bool,
    pub inline: bool,
    pub no_sort: bool,

    // options
    pub lines: Option<usize>,
    pub config: Option<String>,
    pub search: Option<String>,
    pub pool: Option<usize>,
}

/// Arc version of Cfg
pub type Config = Arc<Cfg>;

/// Main configuration structure
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Cfg {
    #[serde(default)]
    pub screen: ScreenConfig,
    #[serde(default)]
    pub initial_query: Option<String>,
    #[serde(default)]
    pub no_sort: bool,

    #[serde(default)]
    pub prompt: PromptConfig,
    #[serde(default)]
    pub gauge: GaugeConfig,
    #[serde(default)]
    pub candidate: CandidateConfig,
    #[serde(default)]
    pub selection: SelectionConfig,

    #[serde(default)]
    pub advanced: AdvancedConfig,
}

/// Configuration constructor
#[derive(Debug, Default)]
pub struct Configurator {
    config: Option<Cfg>,
}

impl Configurator {
    pub fn new() -> Self {
        Self {
            config: Some(Cfg::default()),
        }
    }

    /// Read configuration from default `$HOME/.config/scout.toml` file
    pub fn from_default_file(&mut self) -> &mut Self {
        if let Some(home) = dirs::home_dir() {
            let file_path = home.join(".config/scout.toml");
            let file_path = file_path.to_str();
            if let Some(path) = file_path {
                match self.read_file(path) {
                    Ok(contents) => {
                        self.from_toml(&contents);
                    }
                    Err(_) => log::trace!("Failed to load contents from $HOME/.config/scout.toml"),
                };
            };
        };

        self
    }

    /// Read configuration from the given path
    pub fn from_file<'a>(&'a mut self, file_path: &str) -> &'a mut Self {
        match self.read_file(file_path) {
            Ok(contents) => self.from_toml(&contents),
            Err(err) => panic!("Failed to read file {:?}. Error {:?}", file_path, err),
        }
    }

    /// Parse toml configuration
    pub fn from_toml<'a>(&'a mut self, contents: &str) -> &'a mut Self {
        self.config = toml::from_str(contents).ok();
        self
    }

    /// Set screen configuration size from PTTY
    pub fn from_ptty<'a>(&'a mut self, ptty: &fs::File) -> &'a mut Self {
        if let Some(mut config) = self.config.take() {
            let (cols, rows) =
                terminal_size(ptty.as_raw_fd()).expect("Error getting terminal size");
            config.screen.set_full_size(cols as usize, rows as usize);

            self.config = Some(config);
        }

        self
    }

    /// Set configuration options from command line args
    pub fn from_args<'a>(&'a mut self, args: &Args) -> &'a mut Self {
        if let Some(mut config) = self.config.take() {
            if args.full_screen {
                config.screen.full_mode();
            }

            if args.inline {
                config.screen.inline_mode();
                if let Some(given) = args.lines {
                    config.screen.set_height(given);
                }
            }

            if args.no_sort {
                config.no_sort = true;
            }

            if let Some(q) = &args.search {
                let q = q.to_string();
                config.initial_query = Some(q);
            }

            if let Some(pool) = args.pool {
                config.advanced.set_pool_size(pool);
            }

            self.config = Some(config);
        }

        self
    }

    /// Generate the final Config instance
    pub fn build(&mut self) -> Config {
        match self.config.take() {
            Some(config) => Arc::new(config),
            None => Default::default(),
        }
    }

    fn read_file(&self, file_path: &str) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }
}
