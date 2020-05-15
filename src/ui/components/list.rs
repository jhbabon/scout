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
use unicode_truncate::UnicodeTruncateStr;

#[derive(Debug, Clone)]
struct Item {
    pub candidate: Candidate,
    pub is_selected: bool,
    renderer: Arc<ItemRenderer>,
}

impl Item {
    pub fn new(candidate: &Candidate, renderer: &Arc<ItemRenderer>, is_selected: bool) -> Self {
        Self {
            candidate: candidate.clone(),
            renderer: renderer.clone(),
            is_selected,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.renderer.render(self, f)
    }
}

#[derive(Debug, Clone, Default)]
struct ItemRenderer {
    width: usize,

    candidate_symbol: String,
    candidate_style: Style,
    candidate_style_match: Style,
    candidate_style_symbol: Style,

    selection_symbol: String,
    selection_style: Style,
    selection_style_match: Style,
    selection_style_symbol: Style,
}

impl ItemRenderer {
    fn render(&self, item: &Item, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Use custom truncate from Text
        // TODO: Remove from width space taken by symbol
        let (truncated, _) = item.candidate.text.string.unicode_truncate(self.width);

        let mut symbol = &self.candidate_symbol;
        let mut style = self.candidate_style;
        let mut style_match = self.candidate_style_match;
        let mut style_symbol = self.candidate_style_symbol;

        if item.is_selected {
            symbol = &self.selection_symbol;
            style = self.selection_style;
            style_match = self.selection_style_match;
            style_symbol = self.selection_style_symbol;
        }

        let mut strings: Vec<ANSIString<'_>> = vec![style_symbol.paint(symbol)];
        let mut painted = format_matches(&item.candidate, &truncated, style, style_match);
        strings.append(&mut painted);

        write!(f, "{}{}", clear::CurrentLine, ANSIStrings(&strings))
    }
}

fn format_matches<'a>(
    candidate: &Candidate,
    string: &'a str,
    unmatch_style: Style,
    match_style: Style,
) -> Vec<ANSIString<'a>> {
    let mut pieces = Vec::new();

    // TODO: Redo this using new matches
    pieces.push(unmatch_style.paint(string));

    pieces
}

impl From<&Config> for ItemRenderer {
    fn from(config: &Config) -> Self {
        let width = config.screen.width();

        let candidate_style = config.candidate.style().into();
        let candidate_style_match = config.candidate.style_match().into();
        let candidate_symbol = config.candidate.symbol();
        let candidate_style_symbol = config.candidate.style_symbol().into();

        let selection_style = config.selection.style().into();
        let selection_style_match = config.selection.style_match().into();
        let selection_symbol = config.selection.symbol();
        let selection_style_symbol = config.selection.style_symbol().into();

        Self {
            width,
            candidate_style,
            candidate_style_match,
            candidate_style_symbol,
            candidate_symbol,
            selection_style,
            selection_style_match,
            selection_style_symbol,
            selection_symbol,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct List {
    height: usize,
    offset: usize,
    items: Vec<Item>,
    renderer: Arc<ItemRenderer>,
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
        let renderer = Arc::new(config.into());

        Self {
            height,
            renderer,
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
                let is_selected = index == state.selection_idx();

                Item::new(&candidate, &self.renderer, is_selected)
            })
            .collect();

        Ok(())
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.items.len();
        for (idx, item) in self.items.iter().enumerate() {
            let eol = if idx == (len - 1) { "" } else { "\n" };
            write!(f, "{}{}", item, eol)?;
        }

        Ok(())
    }
}
