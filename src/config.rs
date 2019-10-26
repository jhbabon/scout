use async_std::fs;
use async_std::os::unix::io::AsRawFd;
use serde::{Serialize, Deserialize};
// TODO: Better crate imports
use crate::terminal_size::{terminal_size};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct ScreenConfig {
    pub size: (usize, usize),
    // TODO: Enum?
    pub full: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct Config {
    pub screen: ScreenConfig,
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

    // TODO: Accept actual args from CLI
    pub fn from_args<'a>(&'a mut self, full_screen: bool) -> &'a mut Self {
        if full_screen {
            self.config.screen.full = true;
        } else {
            self.config.screen.full = false;
            self.config.screen.size = (100, 6);
        }

        self
    }

    pub fn build(&self) -> Config {
        self.config
    }
}
