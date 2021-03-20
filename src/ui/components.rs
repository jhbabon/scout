//! UI Component definitions
use crate::common::Result;
use crate::config::Config;
use crate::fuzzy::Candidate;
use crate::state::State;
use crate::ui::painting::{Brush, Canvas, Projector};
use ansi_term::{ANSIString, ANSIStrings, Style};
use std::convert::From;
use std::fmt;
use termion::{clear, cursor};

#[derive(Debug)]
pub struct PromptComponent {
    pub symbol: String,
    pub style: Style,
    pub style_symbol: Style,
    projector: Projector,
}

impl PromptComponent {
    pub fn render(&self, state: &State, canvas: &mut Canvas) -> Result<()> {
        let mut brush = Brush::new(canvas, &self.projector);
        // TODO: Let's try not to transform chars into strings all the time
        // TODO: Maybe use generic fmt::Display in Tile::Filled?
        for ch in self.symbol.chars() {
            brush.draw(ch.into(), self.style_symbol)?;
        }
        for ch in state.query().chars() {
            brush.draw(ch.into(), self.style)?;
        }
        brush.set_cursor()?;
        brush.clear_until_eol()?;

        Ok(())
    }
}

impl From<&Config> for PromptComponent {
    fn from(config: &Config) -> Self {
        // FIXME: The coordinates don't work on inline mode
        let projector = Projector::new((0, 0), config.screen.width(), 1);
        Self {
            symbol: config.prompt.symbol(),
            style: config.prompt.style().into(),
            style_symbol: config.prompt.style_symbol().into(),
            projector,
        }
    }
}

#[derive(Debug)]
pub struct GaugeComponent {
    pub symbol: String,
    pub prefix: String,
    pub style: Style,
    projector: Projector,
}

impl GaugeComponent {
    pub fn render(&self, state: &State, canvas: &mut Canvas) -> Result<()> {
        let mut brush = Brush::new(canvas, &self.projector);
        let current = format!("{}", state.matches().len());
        let total = format!("{}", state.pool_len());

        for ch in self.prefix.chars() {
            brush.draw(ch.into(), self.style)?;
        }

        for ch in current.chars() {
            brush.draw(ch.into(), self.style)?;
        }

        for ch in self.symbol.chars() {
            brush.draw(ch.into(), self.style)?;
        }

        for ch in total.chars() {
            brush.draw(ch.into(), self.style)?;
        }

        brush.clear_until_eol()?;

        Ok(())
    }
}

impl From<&Config> for GaugeComponent {
    fn from(config: &Config) -> Self {
        let projector = Projector::new((0, 1), config.screen.width(), 1);
        Self {
            style: config.gauge.style().into(),
            symbol: config.gauge.symbol(),
            prefix: config.gauge.prefix(),
            projector,
        }
    }
}

#[derive(Debug)]
pub struct ItemStyles {
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

#[derive(Debug)]
pub struct ListComponent {
    pub height: usize,
    pub offset: usize,
    pub candidate_styles: ItemStyles,
    pub selection_styles: ItemStyles,
    projector: Projector,
}

impl ListComponent {
    // TODO: Render Trait
    pub fn render(&self, state: &State, canvas: &mut Canvas) -> Result<()> {
        let mut brush = Brush::new(canvas, &self.projector);
        let lines = self.projector.height();

        let mut items = state
            .matches()
            .iter()
            .enumerate()
            .skip(self.offset)
            .take(lines)
            .peekable();

        while let Some((idx, candidate)) = items.next() {
            let styles = if idx == state.selection_idx() {
                &self.selection_styles
            } else {
                &self.candidate_styles
            };

            for ch in styles.symbol.chars() {
                brush.draw(ch.into(), styles.style_symbol)?;
            }

            for (index, grapheme) in candidate
                .iter()
                .enumerate()
                .take(self.projector.width() - styles.symbol.len())
            {
                if candidate.matches.contains(&index) {
                    brush.draw(grapheme.into(), styles.style_match)?;
                } else {
                    brush.draw(grapheme.into(), styles.style)?;
                }
            }

            brush.clear_until_eol()?;
            brush.new_line()?;
        }

        brush.clear_until_eof()?;

        Ok(())
    }

    pub fn scroll(&mut self, state: &State) {
        let len = self.projector.height();
        let selection = state.selection_idx();

        let top_position = self.offset;
        let last_position = (len + self.offset) - 1;

        // cycle through the list
        if selection > last_position {
            self.offset += selection - last_position;
        } else if selection < top_position {
            self.offset -= top_position - selection;
        };
    }
}

impl From<&Config> for ListComponent {
    fn from(config: &Config) -> Self {
        let offset = 0;
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

        let projector = Projector::new((0, 2), width, height - 2);
        Self {
            height,
            offset,
            candidate_styles,
            selection_styles,
            projector,
        }
    }
}
