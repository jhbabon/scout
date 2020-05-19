pub mod components;
pub mod styling;

use components::*;

use crate::common::Result;
use async_std::fs;
use async_std::os::unix::io::AsRawFd;
use async_std::sync::Arc;
use clap::{value_t, ArgMatches};
use dirs;
use log;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;

use crate::terminal_size::terminal_size;

// Config is shared among tasks, so it makes sense to put it behind an Arc
pub type Config = Arc<Cfg>;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Cfg {
    #[serde(default)]
    pub screen: ScreenConfig,
    #[serde(default)]
    pub initial_query: Option<String>,

    #[serde(default)]
    pub prompt: PromptConfig,
    #[serde(default)]
    pub gauge: GaugeConfig,
    #[serde(default)]
    pub candidate: CandidateConfig,
    #[serde(default)]
    pub selection: SelectionConfig,
}

#[derive(Debug)]
pub struct Configurator {
    config: Option<Cfg>,
}

impl Configurator {
    pub fn new() -> Self {
        Self {
            config: Some(Cfg::default()),
        }
    }

    // Read configuration from default $HOME/.config/scout.toml file
    pub fn from_default_file<'a>(&'a mut self) -> &'a mut Self {
        if let Some(home) = dirs::home_dir() {
            let file_path = home.join(".config/scout.toml");
            let file_path = file_path.to_str();
            if file_path.is_some() {
                match self.read_file(&file_path.unwrap()) {
                    Ok(contents) => {
                        self.from_toml(&contents);
                    }
                    Err(_) => log::trace!("Failed to load contents from $HOME/.config/scout.toml"),
                };
            };
        };

        self
    }

    // Read configuration from the given path
    pub fn from_file<'a>(&'a mut self, file_path: &str) -> &'a mut Self {
        match self.read_file(file_path) {
            Ok(contents) => self.from_toml(&contents),
            Err(_) => panic!("Failed to read file {:?}", file_path),
        }
    }

    fn read_file(&self, file_path: &str) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    // Parse toml configuration
    pub fn from_toml<'a>(&'a mut self, contents: &str) -> &'a mut Self {
        self.config = toml::from_str(contents).ok();
        self
    }

    // Set screen full size from PTTY
    pub fn from_ptty<'a>(&'a mut self, ptty: &fs::File) -> &'a mut Self {
        if let Some(mut config) = self.config.take() {
            let (cols, rows) =
                terminal_size(ptty.as_raw_fd()).expect("Error getting terminal size");
            config.screen.set_full_size(cols as usize, rows as usize);

            self.config = Some(config);
        }

        self
    }

    // Set configuration options from command line args
    pub fn from_args<'a>(&'a mut self, args: &ArgMatches) -> &'a mut Self {
        if let Some(mut config) = self.config.take() {
            if args.is_present("full-screen") {
                config.screen.full_mode();
            }

            if args.is_present("inline") {
                config.screen.inline_mode();
                let given = value_t!(args, "lines", usize).unwrap_or(6);
                config.screen.set_height(given);
            }

            if let Some(q) = args.value_of("search") {
                let q = q.to_string();
                config.initial_query = Some(q);
            }

            self.config = Some(config);
        }

        self
    }

    pub fn build(&mut self) -> Config {
        match self.config.take() {
            Some(config) => Arc::new(config),
            None => Default::default(),
        }
    }
}
