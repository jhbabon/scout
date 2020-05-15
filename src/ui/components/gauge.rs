use super::component::*;

use crate::common::Result;
use crate::config::Config;
use crate::state::State;
use ansi_term::Style;
use std::convert::From;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Gauge {
    symbol: String,
    prefix: String,
    style: Style,
    current: usize,
    total: usize,
}

impl From<&Config> for Gauge {
    fn from(config: &Config) -> Self {
        let style = config.gauge.style().into();
        let symbol = config.gauge.symbol();
        let prefix = config.gauge.prefix();

        Self {
            style,
            symbol,
            prefix,
            ..Default::default()
        }
    }
}

impl Component for Gauge {
    fn update(&mut self, state: &State) -> Result<()> {
        self.current = state.matches().len();
        self.total = state.pool_len();

        Ok(())
    }
}

impl fmt::Display for Gauge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = format!(
            "{}{}{}{}",
            self.prefix, self.current, self.symbol, self.total
        );

        write!(f, "{}", self.style.paint(&display))
    }
}
