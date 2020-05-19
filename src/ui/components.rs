//! UI Component definitions
//!
//! A Component is basically a type that has all the styling and basic information to print to the
//! screen correctly, but it doesn't have the actual data to print, just how it should look.
//! It will then delegate the actual printing to a Renderer, a type that given
//! the component styling information and the current `State` will know how to print to the screen.
//! A Renderer only needs to implement the `fmt::Display` trait.
//!
//! This two steps process for printing is done so we only need the state information while
//! printing and not before, which means we can use references to get the data and prevent any
//! extra data allocation from the state to the components.
use crate::config::Config;
use crate::fuzzy::Candidate;
use crate::state::State;
use ansi_term::{ANSIString, ANSIStrings, Style};
use std::convert::From;
use std::fmt;
use termion::{clear, cursor};

pub trait Render<'r, R>
where
    R: fmt::Display + 'r,
{
    fn render(&'r self, state: &'r State) -> R;
}

#[derive(Debug)]
pub struct PromptRenderer<'r> {
    prompt: &'r PromptComponent,
    state: &'r State,
}

impl<'r> fmt::Display for PromptRenderer<'r> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = self.prompt.style_symbol.paint(&self.prompt.symbol);
        let query = self.prompt.style.paint(self.state.query());
        let left_moves = self.state.cursor_until_end() as u16;

        if left_moves == 0 {
            write!(f, "{}{}", symbol, query)
        } else {
            write!(f, "{}{}{}", symbol, query, cursor::Left(left_moves))
        }
    }
}

#[derive(Debug)]
pub struct PromptComponent {
    pub symbol: String,
    pub style: Style,
    pub style_symbol: Style,
}

impl<'r> Render<'r, PromptRenderer<'r>> for PromptComponent {
    fn render(&'r self, state: &'r State) -> PromptRenderer<'r> {
        PromptRenderer {
            prompt: self,
            state,
        }
    }
}

impl From<&Config> for PromptComponent {
    fn from(config: &Config) -> Self {
        Self {
            symbol: config.prompt.symbol(),
            style: config.prompt.style().into(),
            style_symbol: config.prompt.style_symbol().into(),
        }
    }
}

#[derive(Debug)]
pub struct GaugeRenderer<'r> {
    gauge: &'r GaugeComponent,
    state: &'r State,
}

impl<'r> fmt::Display for GaugeRenderer<'r> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let current = self.state.matches().len();
        let total = self.state.pool_len();

        write!(
            f,
            "{}{}{}{}{}{}",
            self.gauge.style.prefix(),
            self.gauge.prefix,
            current,
            self.gauge.symbol,
            total,
            self.gauge.style.suffix()
        )
    }
}

#[derive(Debug)]
pub struct GaugeComponent {
    pub symbol: String,
    pub prefix: String,
    pub style: Style,
}

impl From<&Config> for GaugeComponent {
    fn from(config: &Config) -> Self {
        Self {
            style: config.gauge.style().into(),
            symbol: config.gauge.symbol(),
            prefix: config.gauge.prefix(),
        }
    }
}

impl<'r> Render<'r, GaugeRenderer<'r>> for GaugeComponent {
    fn render(&'r self, state: &'r State) -> GaugeRenderer<'r> {
        GaugeRenderer { gauge: self, state }
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
pub struct ListRenderer<'r> {
    list: &'r ListComponent,
    state: &'r State,
}

impl<'r> ListRenderer<'r> {
    pub fn len(&'r self) -> usize {
        let len;
        let lines = self.list.height - 2;
        let matches_len = self.state.matches().len();

        if matches_len >= self.list.offset {
            len = matches_len - self.list.offset;
        } else {
            len = self.list.offset - matches_len;
        }

        if len >= lines {
            lines
        } else {
            len
        }
    }
}

impl<'r> fmt::Display for ListRenderer<'r> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self.list.height - 2;

        let mut items = self
            .state
            .matches()
            .iter()
            .enumerate()
            .skip(self.list.offset)
            .take(lines)
            .peekable();

        while let Some((idx, candidate)) = items.next() {
            let eol = if items.peek().is_none() { "" } else { "\n" };

            let styles = if idx == self.state.selection_idx() {
                &self.list.selection_styles
            } else {
                &self.list.candidate_styles
            };

            render_item(f, candidate, styles, eol)?
        }

        Ok(())
    }
}

fn render_item(
    f: &mut fmt::Formatter<'_>,
    candidate: &Candidate,
    styles: &ItemStyles,
    eol: &str,
) -> fmt::Result {
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

#[derive(Debug)]
pub struct ListComponent {
    pub height: usize,
    pub offset: usize,
    pub candidate_styles: ItemStyles,
    pub selection_styles: ItemStyles,
}

impl ListComponent {
    pub fn scroll(&mut self, state: &State) {
        let len = self.height - 2;

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

        Self {
            height,
            offset,
            candidate_styles,
            selection_styles,
        }
    }
}

impl<'r> Render<'r, ListRenderer<'r>> for ListComponent {
    fn render(&'r self, state: &'r State) -> ListRenderer<'r> {
        ListRenderer { list: self, state }
    }
}
