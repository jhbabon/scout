use std::fmt::{self, Write};
use termion::{clear,cursor,style};
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

impl Layout {
    pub fn new(config: &Config) -> Self {
        let display = None;
        let size = config.screen.size;
        let offset = 0;

        Self { display, size, offset }
    }

    pub fn draw(&mut self, state: &State) -> Result<()> {
        let mut display = String::new();

        match state.last_update() {
            StateUpdate::Query => {
                let prompt = self.draw_prompt(&state)?;
                write!(&mut display, "{}", prompt)?;
            },
            _ => {
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
            "{}\r$ {}",
            clear::CurrentLine,
            state.query()
        );

        Ok(prompt)
    }

    fn draw_list(&mut self, state: &State) -> Result<String> {
        let mut display = String::new();

        let counter = format!("{}  {}/{}", clear::CurrentLine, state.matches().len(), state.pool_len());

        let invert = format!("{}", style::Invert);
        let no_invert = format!("{}", style::NoInvert);

        let (width, _) = self.size;
        let line_len = width - 2;
        let (offset, lines) = self.scroll(&state);
        let mut list: Vec<String> = state.matches()
            .iter()
            .cloned()
            .enumerate()
            .skip(offset)
            .take(lines)
            .map(|(idx, c)| (idx, c.text))
            .map(|(index, candidate)| {
                // FIXME: Do not pad, only truncate
                let truncated = candidate.unicode_pad(line_len, Alignment::Left, true);
                if index == state.selection_idx() {
                    format!("{}{}> {}{}", clear::CurrentLine, invert, truncated, no_invert)
                } else {
                    format!("{}  {}", clear::CurrentLine, truncated)
                }

            })
            .collect();

        // The counter is another element of the list
        list.insert(0, counter);

        write!(
            &mut display,
            "{}\r{}{}{}",
            cursor::Down(1),
            list.join("\n"),
            clear::AfterCursor,
            // We always need to reprint the prompt after
            // going up to set the cursor in the last
            // position
            cursor::Up(list.len() as u16),
        )?;

        Ok(display)
    }

    fn scroll(&mut self, state: &State) -> (usize, usize) {
        let (_, height) = self.size;
        let lines_len = height - 2;

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
