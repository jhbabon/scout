pub mod components;
pub mod styling;

use components::*;

use async_std::fs;
use async_std::os::unix::io::AsRawFd;
use clap::{value_t, ArgMatches};
use serde::Deserialize;
use toml;

use crate::terminal_size::terminal_size;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
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
    config: Option<Config>,
}

impl Configurator {
    pub fn new() -> Self {
        Self {
            config: Some(Config::default()),
        }
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
            Some(config) => config,
            None => Default::default(),
        }
    }
}
