use super::component::*;

use crate::common::Result;
use crate::config::Config;
use crate::fuzzy::Candidate;
use crate::state::State;
use ansi_term::{ANSIString, ANSIStrings, Style};
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

#[derive(Debug, Clone, Default)]
pub struct List {
    height: usize,
    offset: usize,
    items: Vec<(Candidate, bool)>,
    candidate_styles: ItemStyles,
    selection_styles: ItemStyles,
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
            candidate_styles,
            selection_styles,
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
                    (candidate.clone(), true)
                } else {
                    (candidate.clone(), false)
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
            let (candidate, is_selected) = item;
            let styles = if *is_selected { &self.selection_styles } else { &self.candidate_styles };
            render_item(candidate, styles, eol, f)?
        }

        Ok(())
    }
}

fn render_item(candidate: &Candidate, styles: &ItemStyles, eol: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let symbol = &styles.symbol;
    let style = &styles.style;
    let style_match = &styles.style_match;
    let style_symbol = &styles.style_symbol;

    let mut strings: Vec<ANSIString<'_>> = vec![style_symbol.paint(symbol)];
    let mut painted: Vec<ANSIString<'_>> = candidate
        .iter()
        .enumerate()
        .take(styles.width - symbol.len())
        .map(|(index, grapheme)| {
            if candidate.matches.contains(&index) {
                style_match.paint(grapheme)
            } else {
                style.paint(grapheme)
            }
        })
    .collect();

    strings.append(&mut painted);

    // ANSIStrings already takes care of reducing the number of escape
    // sequences that will be printed to the terminal
    write!(f, "{}{}{}", clear::CurrentLine, ANSIStrings(&strings), eol)
}
