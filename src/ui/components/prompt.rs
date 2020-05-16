use super::component::*;

use crate::common::Result;
use crate::config::Config;
use crate::state::State;
use ansi_term::Style;
use std::convert::From;
use std::fmt;
use termion::cursor;

#[derive(Debug, Clone, Default)]
pub struct Prompt {
    left_moves: u16,
    symbol: String,
    query: String,
    style: Style,
    style_symbol: Style,
}

impl From<&Config> for Prompt {
    fn from(config: &Config) -> Self {
        let symbol = config.prompt.symbol();
        let style = config.prompt.style().into();
        let style_symbol = config.prompt.style_symbol().into();

        Self {
            symbol,
            style,
            style_symbol,
            ..Default::default()
        }
    }
}

impl Component for Prompt {
    fn update(&mut self, state: &State) -> Result<()> {
        self.query = state.query();
        self.left_moves = state.cursor_until_end() as u16;

        Ok(())
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.left_moves == 0 {
            write!(
                f,
                "{}{}",
                self.style_symbol.paint(&self.symbol),
                self.style.paint(&self.query)
            )
        } else {
            write!(
                f,
                "{}{}{}",
                self.style_symbol.paint(&self.symbol),
                self.style.paint(&self.query),
                cursor::Left(self.left_moves)
            )
        }
    }
}
