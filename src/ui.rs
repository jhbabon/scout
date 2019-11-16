use crate::common::Result;
use crate::config::Config;
use crate::fuzzy::Candidate;
use crate::state::{State, StateUpdate};
use ansi_term::{ANSIString, ANSIStrings, Style};
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use std::fmt;
use termion::{clear, cursor};
use unicode_truncate::UnicodeTruncateStr;

const ALTERNATE_SCREEN: &'static str = csi!("?1049h");
const MAIN_SCREEN: &'static str = csi!("?1049l");

trait Component {
    fn render(&mut self, state: &State) -> Result<()>;
}

#[derive(Debug, Clone, Default)]
struct Prompt {
    symbol: String,
    query: String,
}

// TODO: From Config
impl Prompt {
    fn new(_config: &Config) -> Self {
        let symbol = "$".into();
        let query = "".into();

        Self { symbol, query }
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
        write!(f, "{} {}", self.symbol, self.query)
    }
}

#[derive(Debug, Clone, Default)]
struct Meter {
    current: usize,
    total: usize,
}

// TODO: From Config
impl Meter {
    fn new(_config: &Config) -> Self {
        let current = 0;
        let total = 0;

        Self { current, total }
    }
}

impl Component for Meter {
    fn render(&mut self, state: &State) -> Result<()> {
        self.current = state.matches().len();
        self.total = state.pool_len();

        Ok(())
    }
}

impl fmt::Display for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  {}/{}", self.current, self.total)
    }
}

#[derive(Debug, Clone)]
enum Item {
    Choice(Candidate, usize),
    Selected(Candidate, usize),
}

impl Item {
    fn new(width: usize, candidate: Candidate, selected: bool) -> Self {
        if selected {
            Self::Selected(candidate, width)
        } else {
            Self::Choice(candidate, width)
        }
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

    if let Some(result) = &candidate.score_match {
        let mut last_end = 0;

        for &(start, len) in &result.continuous_matches() {
            // Take piece between last match and this match.
            pieces.push(
                unmatch_style.paint(
                    string
                        .chars()
                        .skip(last_end)
                        .take(start - last_end)
                        .collect::<String>(),
                ),
            );
            // Add actual match.
            pieces
                .push(match_style.paint(string.chars().skip(start).take(len).collect::<String>()));
            last_end = start + len;
        }

        // If there's characters left after the last match, make sure to append them.
        if last_end != string.len() {
            pieces.push(
                unmatch_style.paint(
                    string
                        .chars()
                        .skip(last_end)
                        .take_while(|_| true)
                        .collect::<String>(),
                ),
            );
        }
    } else {
        pieces.push(unmatch_style.paint(string));
    };

    pieces
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Choice(candidate, width) => {
                let (truncated, _) = candidate.text.unicode_truncate(*width);

                let painted =
                    format_matches(&candidate, &truncated, Style::new(), Style::new().bold());

                write!(f, "{}  {}", clear::CurrentLine, ANSIStrings(&painted),)
            }
            Self::Selected(candidate, width) => {
                let (truncated, _) = candidate.text.unicode_truncate(*width);

                let mut strings: Vec<ANSIString<'_>> = vec![Style::new().reverse().paint("> ")];
                let mut painted = format_matches(
                    &candidate,
                    &truncated,
                    Style::new().reverse(),
                    Style::new().reverse().bold(),
                );
                strings.append(&mut painted);

                write!(f, "{}{}", clear::CurrentLine, ANSIStrings(&strings),)
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct List {
    size: (usize, usize),
    offset: usize,
    items: Vec<Item>,
}

// TODO: From Config
impl List {
    fn new(config: &Config) -> Self {
        let size = config.screen.size;
        let offset = 0;
        let items = vec![];

        Self {
            size,
            offset,
            items,
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
                let selected = index == state.selection_idx();

                Item::new(line_len, candidate, selected)
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
    meter: Meter,
    list: List,
}

impl<W: io::Write + Send + Unpin + 'static> Layout<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let size = config.screen.size;
        let mode = if config.screen.full {
            Mode::Full
        } else {
            let (_, height) = size;
            Mode::Inline(height)
        };

        let prompt = Prompt::new(config);
        let meter = Meter::new(config);
        let list = List::new(config);

        let mut layout = Self {
            mode,
            writer,
            prompt,
            meter,
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
                self.meter.render(state)?;
                self.list.render(state)?;

                let list_len = self.list.len() as u16;

                // Only add a new line if we are going to print items
                let meter_separator = if list_len == 0 { "" } else { "\n" };

                let display = format!(
                    "{}{}\r{}{}{}{}{}{}\r{}",
                    cursor::Down(1),
                    clear::CurrentLine,
                    self.meter,
                    meter_separator,
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
