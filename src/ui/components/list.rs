use super::component::*;

use crate::common::Result;
use crate::config::Config;
use crate::fuzzy::Candidate;
use crate::state::State;
use ansi_term::{ANSIString, ANSIStrings, Style};
use async_std::sync::Arc;
use std::convert::From;
use std::fmt;
use termion::clear;

#[derive(Debug, Clone, Default)]
struct ItemStyles {
    pub width: usize,
    pub symbol: String,
    pub style: Style,
    pub style_match: Style,
    pub style_symbol: Style,
}

impl ItemStyles {
    fn new(
        width: usize,
        symbol: String,
        style: Style,
        style_match: Style,
        style_symbol: Style,
    ) -> Self {
        Self {
            width,
            symbol,
            style,
            style_match,
            style_symbol,
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    candidate: Candidate,
    styles: Arc<ItemStyles>,
}

impl Item {
    pub fn new(candidate: &Candidate, styles: &Arc<ItemStyles>) -> Self {
        Self {
            candidate: candidate.clone(),
            styles: styles.clone(),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = &self.styles.symbol;
        let style = &self.styles.style;
        let style_match = &self.styles.style_match;
        let style_symbol = &self.styles.style_symbol;

        let mut strings: Vec<ANSIString<'_>> = vec![style_symbol.paint(symbol)];
        let mut painted: Vec<ANSIString<'_>> = self
            .candidate
            .iter()
            .enumerate()
            .take(self.styles.width - symbol.len())
            .map(|(index, grapheme)| {
                if self.candidate.matches.contains(&index) {
                    style_match.paint(grapheme)
                } else {
                    style.paint(grapheme)
                }
            })
            .collect();

        strings.append(&mut painted);

        // ANSIStrings already takes care of reducing the number of escape
        // sequences that will be printed to the terminal
        write!(f, "{}{}", clear::CurrentLine, ANSIStrings(&strings))
    }
}

#[derive(Debug, Clone, Default)]
pub struct List {
    height: usize,
    offset: usize,
    items: Vec<Item>,
    candidate_styles: Arc<ItemStyles>,
    selection_styles: Arc<ItemStyles>,
}

impl List {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    fn scroll(&mut self, state: &State) -> (usize, usize) {
        let len = self.height - 2;

        let selection = state.selection_idx();

        let top_position = self.offset;
        let last_position = (len + self.offset) - 1;

        // cycle through the list
        if selection > last_position {
            self.offset += selection - last_position;
        } else if selection < top_position {
            self.offset -= top_position - selection;
        }

        (self.offset, len)
    }
}

impl From<&Config> for List {
    fn from(config: &Config) -> Self {
        let height = config.screen.height();
        let width = config.screen.width();

        let candidate_styles = ItemStyles::new(
            width,
            config.candidate.symbol(),
            config.candidate.style().into(),
            config.candidate.style_match().into(),
            config.candidate.style_symbol().into(),
        );

        let selection_styles = ItemStyles::new(
            width,
            config.selection.symbol(),
            config.selection.style().into(),
            config.selection.style_match().into(),
            config.selection.style_symbol().into(),
        );

        Self {
            height,
            candidate_styles: Arc::new(candidate_styles),
            selection_styles: Arc::new(selection_styles),
            ..Default::default()
        }
    }
}

impl Component for List {
    fn update(&mut self, state: &State) -> Result<()> {
        let (offset, lines) = self.scroll(state);

        self.items = state
            .matches()
            .iter()
            .enumerate()
            .skip(offset)
            .take(lines)
            .map(|(index, candidate)| {
                if index == state.selection_idx() {
                    Item::new(&candidate, &self.selection_styles)
                } else {
                    Item::new(&candidate, &self.candidate_styles)
                }
            })
            .collect();

        Ok(())
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.items.len();
        let last = if len < 1 { 0 } else { len - 1 };
        for (idx, item) in self.items.iter().enumerate() {
            let eol = if idx == last { "" } else { "\n" };
            write!(f, "{}{}", item, eol)?;
        }

        Ok(())
    }
}
