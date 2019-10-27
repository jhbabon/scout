use log::debug;
use async_std::sync::Arc;
use std::fmt::{self, Write};
use sublime_fuzzy::format_simple;
use termion;
use unicode_truncate::UnicodeTruncateStr;
use unicode_truncate::Alignment;
use crate::config::Config;
use crate::common::Result;
use crate::state::{State,StateUpdate};

#[derive(Debug,Clone)]
pub struct Layout {
    display: Option<String>,
    size: (usize, usize),
    offset: usize,
}

// FIXME: Make layout rendering more efficient
//  * Maybe use a buffwriter in the renderer?
//  * No writeln! macros
//  * Only redraw lines that need change?
//  * Use ansi_term for colors, etc
impl Layout {
    pub fn new(config: &Config) -> Self {
        let display = None;
        let size = config.screen.size;
        let offset = 0;

        Self { display, size, offset }
    }

    pub fn draw(&mut self, state: &State) -> Result<()> {
        let mut display = String::new();

        match state.last_update {
            StateUpdate::Query => {
                let prompt = self.draw_prompt(&state)?;
                write!(&mut display, "{}", prompt)?;
            },
            StateUpdate::Matches | StateUpdate::Selection => {
                let list = self.draw_list(&state)?;
                write!(&mut display, "{}", list)?;
            },
            StateUpdate::All => {
                let list = self.draw_list(&state)?;
                let prompt = self.draw_prompt(&state)?;
                write!(&mut display, "{}{}", list, prompt)?;
            },
        }

        self.display = Some(display);

        Ok(())
    }

    fn draw_prompt(&mut self, state: &State) -> Result<String> {
        let prompt = format!(
            "{}\r> {}",
            termion::clear::CurrentLine,
            state.query()
        );

        Ok(prompt)
    }

    fn draw_list(&mut self, state: &State) -> Result<String> {
        let mut display = String::new();

        // write!(&mut display, "{}", termion::clear::AfterCursor)?;
        write!(&mut display, "{}", termion::cursor::Save)?;

        let (width, _) = self.size;
        let line_len = width - 2;
        let (offset, lines) = self.scroll(&state);
        let list: Vec<String> = state.matches
            .iter()
            .cloned()
            .enumerate()
            .skip(offset)
            .take(lines)
            .map(|(idx, c)| {
                if let Some(score_match) = c.score_match {
                    (idx, Arc::new(format_simple(&score_match, &c.string, "", "")))
                } else {
                    (idx, c.string)
                }
            })
            .map(|(index, candidate)| {
                let mut selected = " ";
                if index == state.selection_idx() {
                    selected = ">";
                }

                format!("{} {}", selected, candidate.unicode_pad(line_len, Alignment::Left, true))
            })
            .collect();

        write!(
            &mut display,
            "{}\r{}{}{}",
            termion::cursor::Down(1),
            list.join("\n"),
            termion::clear::AfterCursor,
            termion::cursor::Restore,
        )?;

        Ok(display)
    }

    fn scroll(&mut self, state: &State) -> (usize, usize) {
        let (_, height) = self.size;
        let lines_len = height - 1;

        let selection = state.selection_idx();

        let top_position = self.offset;
        let last_position = (lines_len + self.offset) - 1;

        if selection > last_position {
            self.offset += selection - last_position;
        } else if selection < top_position {
            self.offset -= top_position - selection;
        }

        (self.offset, lines_len)
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.display {
            None => write!(f, ""),
            Some(display) => write!(f, "{}", display),
        }
    }
}
