use super::component::*;

use crate::common::Result;
use crate::config::Config;
use crate::state::State;
use ansi_term::Style;
use std::convert::From;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Prompt {
    symbol: String,
    query: String,
    style: Style,
    style_symbol: Style,
}

impl From<&Config> for Prompt {
    fn from(config: &Config) -> Self {
        let symbol = config.prompt.symbol();
        let query = "".into();
        let style = config.prompt.style().into();
        let style_symbol = config.prompt.style_symbol().into();

        Self {
            symbol,
            query,
            style,
            style_symbol,
        }
    }
}

impl Component for Prompt {
    fn update(&mut self, state: &State) -> Result<()> {
        self.query = state.query();

        Ok(())
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.style_symbol.paint(&self.symbol),
            self.style.paint(&self.query)
        )
    }
}
