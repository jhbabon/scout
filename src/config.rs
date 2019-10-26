use async_std::fs;
use async_std::os::unix::io::AsRawFd;
use serde::{Serialize, Deserialize};
use clap::{ArgMatches,value_t};
// TODO: Better crate imports
use crate::terminal_size::{terminal_size};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ScreenConfig {
    pub size: (usize, usize),
    // TODO: Enum?
    pub full: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub screen: ScreenConfig,
    pub initial_query: Option<String>,
}

#[derive(Debug)]
pub struct Configurator {
    config: Config,
}

impl Configurator {
    pub fn new() -> Self {
        Self { config: Config::default() }
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
