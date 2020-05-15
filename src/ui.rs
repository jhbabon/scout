use crate::common::Result;
use crate::config::{styling, Config};
use crate::fuzzy::Candidate;
use crate::state::{State, StateUpdate};
use ansi_term::{ANSIString, ANSIStrings, Color, Style};
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use std::convert::Into;
use std::fmt;
use termion::{clear, cursor};
use unicode_truncate::UnicodeTruncateStr;

const ALTERNATE_SCREEN: &'static str = csi!("?1049h");
const MAIN_SCREEN: &'static str = csi!("?1049l");

impl Into<Color> for styling::Color {
    fn into(self) -> Color {
        match self {
            styling::Color::Black => Color::Black,
            styling::Color::Red => Color::Red,
            styling::Color::Yellow => Color::Yellow,
            styling::Color::Green => Color::Green,
            styling::Color::Blue => Color::Blue,
            styling::Color::Purple => Color::Purple,
            styling::Color::Cyan => Color::Cyan,
            styling::Color::White => Color::White,
            styling::Color::Fixed(n) => Color::Fixed(n),
            styling::Color::RGB(r, g, b) => Color::RGB(r, g, b),
        }
    }
}

impl Into<Style> for styling::Style {
    fn into(self) -> Style {
        self.into_iter().fold(Style::default(), |acc, rule| match rule {
            styling::Rule::Reset => Style::default(),
            styling::Rule::Underline => acc.underline(),
            styling::Rule::Strikethrough => acc.strikethrough(),
            styling::Rule::Reverse => acc.reverse(),
            styling::Rule::Bold => acc.bold(),
            styling::Rule::Italic => acc.italic(),
            styling::Rule::Dimmed => acc.dimmed(),
            styling::Rule::Fg(color) => acc.fg(color.into()),
            styling::Rule::Bg(color) => acc.on(color.into()),
        })
    }
}

trait Component {
    fn render(&mut self, state: &State) -> Result<()>;
}

#[derive(Debug, Clone, Default)]
struct Prompt {
    symbol_style: Style,
    symbol: String,
    query_style: Style,
    query: String,
}

impl Prompt {
    fn new(config: &Config) -> Self {
        let symbol_style = config.prompt.style_symbol().into();
        let symbol = config.prompt.symbol();
        let query_style = config.prompt.style().into();
        let query = "".into();

        Self {
            symbol_style,
            symbol,
            query_style,
            query,
        }
    }
}

impl Component for Prompt {
    fn render(&mut self, state: &State) -> Result<()> {
        self.query = state.query();

        Ok(())
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.symbol_style.paint(&self.symbol),
            self.query_style.paint(&self.query)
        )
    }
}

#[derive(Debug, Clone, Default)]
struct Counter {
    style: Style,
    symbol: String,
    prefix: String,
    current: usize,
    total: usize,
}

// TODO: Rename to Gauge
impl Counter {
    fn new(config: &Config) -> Self {
        let style = config.gauge.style().into();
        let symbol = config.gauge.symbol();
        let prefix = config.gauge.prefix();
        let current = 0;
        let total = 0;

        Self {
            style,
            symbol,
            prefix,
            current,
            total,
        }
    }
}

impl Component for Counter {
    fn render(&mut self, state: &State) -> Result<()> {
        self.current = state.matches().len();
        self.total = state.pool_len();

        Ok(())
    }
}

impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = format!("{}{}{}{}", self.prefix, self.current, self.symbol, self.total);

        write!(f, "{}", self.style.paint(&display))
    }
}

#[derive(Debug, Clone)]
struct Item {
    width: usize,
    is_selected: bool,
    candidate: Candidate,

    item_symbol: String,
    item_style: Style,
    item_match_style: Style,
    item_bullet_style: Style,

    selection_symbol: String,
    selection_style: Style,
    selection_match_style: Style,
    selection_bullet_style: Style,
}

impl Item {
    // TODO: Maybe is better to have a ItemBuilder that does the StyleConfig.into once
    // and creates a new Item per new candidate
    fn new(config: &Config, width: usize, candidate: Candidate, is_selected: bool) -> Self {
        let item_style = config.candidate.style().into();
        let item_match_style = config.candidate.style_match().into();
        let item_symbol = config.candidate.symbol();
        let item_bullet_style = config.candidate.style_symbol().into();

        let selection_style = config.selection.style().into();
        let selection_match_style = config.selection.style_match().into();
        let selection_symbol = config.selection.symbol();
        let selection_bullet_style = config.selection.style_symbol().into();

        Self {
            width,
            candidate,
            is_selected,
            item_style,
            item_match_style,
            item_bullet_style,
            item_symbol,
            selection_style,
            selection_match_style,
            selection_bullet_style,
            selection_symbol,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Maybe is better to add padding to the whole line than just the text?
        // If I use unicode_segmentation to get graphemes I can reuse that here to get
        // the truncation
        let (truncated, _) = self.candidate.text.string.unicode_truncate(self.width);

        let mut indicator = &self.item_symbol;
        let mut style = self.item_style;
        let mut match_style = self.item_match_style;
        let mut symbol_style = self.item_bullet_style;

        if self.is_selected {
            indicator = &self.selection_symbol;
            style = self.selection_style;
            match_style = self.selection_match_style;
            symbol_style = self.selection_bullet_style;
        }

        let mut strings: Vec<ANSIString<'_>> = vec![symbol_style.paint(indicator)];
        let mut painted = format_matches(&self.candidate, &truncated, style, match_style);
        strings.append(&mut painted);

        write!(f, "{}{}", clear::CurrentLine, ANSIStrings(&strings),)
    }
}

// Adaptation of the original sublime_fuzzy::format_simple function
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

#[derive(Debug, Clone, Default)]
struct List {
    size: (usize, usize),
    offset: usize,
    items: Vec<Item>,
    config: Config,
}

impl List {
    fn new(config: &Config) -> Self {
        let size = config.screen.size();
        let offset = 0;
        let items = vec![];

        Self {
            size,
            offset,
            items,
            config: config.clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    fn scroll(&mut self, state: &State) -> (usize, usize) {
        let (_, height) = self.size;
        let len = height - 2;

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

impl Component for List {
    fn render(&mut self, state: &State) -> Result<()> {
        let (width, _) = self.size;
        let line_len = width - 2;
        let (offset, lines) = self.scroll(state);

        self.items = state
            .matches()
            .iter()
            .cloned()
            .enumerate()
            .skip(offset)
            .take(lines)
            .map(|(index, candidate)| {
                let is_selected = index == state.selection_idx();

                Item::new(&self.config, line_len, candidate, is_selected)
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

#[derive(Debug, Clone)]
enum Mode {
    Full,
    Inline(usize),
}

impl Mode {
    pub fn setup(&self) -> Option<String> {
        let setup = match self {
            Self::Full => format!("{}{}", ALTERNATE_SCREEN, cursor::Goto(1, 1)),
            Self::Inline(height) => {
                let room = std::iter::repeat("\n")
                    .take(*height)
                    .collect::<Vec<&str>>()
                    .join("");

                let up = *height as u16;

                format!("{}{}\r", room, cursor::Up(up))
            }
        };

        Some(setup)
    }

    pub fn teardown(&self) -> Option<String> {
        let teardown = match self {
            Self::Full => MAIN_SCREEN.to_string(),
            Self::Inline(_) => format!("{}{}{}", clear::CurrentLine, clear::AfterCursor, "\r"),
        };

        Some(teardown)
    }
}

#[derive(Debug, Clone)]
pub struct Layout<W: io::Write + Send + Unpin + 'static> {
    mode: Mode,
    writer: W,
    prompt: Prompt,
    counter: Counter,
    list: List,
}

impl<W: io::Write + Send + Unpin + 'static> Layout<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let size = config.screen.size();
        let mode = if config.screen.is_full() {
            Mode::Full
        } else {
            let (_, height) = size;
            Mode::Inline(height)
        };

        let prompt = Prompt::new(config);
        let counter = Counter::new(config);
        let list = List::new(config);

        let mut layout = Self {
            mode,
            writer,
            prompt,
            counter,
            list,
        };

        if let Some(setup) = layout.mode.setup() {
            layout.write(&setup).await?;
        }

        Ok(layout)
    }

    pub async fn render(&mut self, state: &State) -> Result<()> {
        match state.last_update() {
            StateUpdate::Query => {
                self.prompt.render(state)?;

                let display = format!("{}\r{}", clear::CurrentLine, self.prompt);
                self.write(&display).await?;
            }
            _ => {
                self.prompt.render(state)?;
                self.counter.render(state)?;
                self.list.render(state)?;

                let list_len = self.list.len() as u16;

                // Only add a new line if we are going to print items
                let counter_separator = if list_len == 0 { "" } else { "\n" };

                let display = format!(
                    "{}{}\r{}{}{}{}{}{}\r{}",
                    cursor::Down(1),
                    clear::CurrentLine,
                    self.counter,
                    counter_separator,
                    self.list,
                    clear::AfterCursor,
                    // We always need to reprint the prompt after
                    // going up to set the cursor in the last
                    // position
                    cursor::Up(list_len + 1),
                    clear::CurrentLine,
                    self.prompt,
                );

                self.write(&display).await?;
            }
        }

        Ok(())
    }

    async fn write(&mut self, display: &str) -> Result<()> {
        self.writer.write_all(display.as_bytes()).await?;
        self.writer.flush().await?;

        Ok(())
    }
}

impl<W: io::Write + Send + Unpin + 'static> Drop for Layout<W> {
    fn drop(&mut self) {
        task::block_on(async {
            if let Some(teardown) = self.mode.teardown() {
                self.write(&teardown)
                    .await
                    .expect("Error writing to output");
            }
        });
    }
}
