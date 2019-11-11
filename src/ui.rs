use crate::common::Result;
use crate::config::Config;
use crate::state::{State, StateUpdate};
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use std::fmt::Write;
use termion::{clear, cursor, style};
use unicode_truncate::UnicodeTruncateStr;

const ALTERNATE_SCREEN: &'static str = csi!("?1049h");
const MAIN_SCREEN: &'static str = csi!("?1049l");

#[derive(Debug, Clone)]
enum Screen {
    Full,
    Inline(usize),
}

impl Screen {
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
    size: (usize, usize),
    offset: usize,
    screen: Screen,
    writer: W,
}

impl<W: io::Write + Send + Unpin + 'static> Layout<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let size = config.screen.size;
        let offset = 0;
        let screen = if config.screen.full {
            Screen::Full
        } else {
            let (_, height) = size;
            Screen::Inline(height)
        };

        let mut layout = Self {
            size,
            offset,
            screen,
            writer,
        };

        if let Some(setup) = layout.screen.setup() {
            layout.write(&setup).await?;
        }

        Ok(layout)
    }

    pub async fn write<D: std::fmt::Display>(&mut self, display: &D) -> Result<()> {
        let rendered = format!("{}", display,);
        self.writer.write_all(rendered.as_bytes()).await?;
        self.writer.flush().await?;

        Ok(())
    }

    pub async fn render(&mut self, state: &State) -> Result<()> {
        let d = self.draw(state)?;

        self.write(&d).await?;

        Ok(())
    }

    pub fn draw(&mut self, state: &State) -> Result<String> {
        let mut display = String::new();

        match state.last_update() {
            StateUpdate::Query => {
                let prompt = self.draw_prompt(&state)?;
                write!(&mut display, "{}", prompt)?;
            }
            _ => {
                let list = self.draw_list(&state)?;
                let prompt = self.draw_prompt(&state)?;
                write!(&mut display, "{}{}", list, prompt)?;
            }
        }

        Ok(display)
    }

    fn draw_prompt(&mut self, state: &State) -> Result<String> {
        let prompt = format!("{}\r$ {}", clear::CurrentLine, state.query());

        Ok(prompt)
    }

    fn draw_list(&mut self, state: &State) -> Result<String> {
        let mut display = String::new();

        let counter = format!(
            "{}  {}/{}",
            clear::CurrentLine,
            state.matches().len(),
            state.pool_len()
        );

        let invert = format!("{}", style::Invert);
        let no_invert = format!("{}", style::NoInvert);

        let (width, _) = self.size;
        let line_len = width - 2;
        let (offset, lines) = self.scroll(&state);
        let mut list: Vec<String> = state
            .matches()
            .iter()
            .cloned()
            .enumerate()
            .skip(offset)
            .take(lines)
            .map(|(idx, c)| (idx, c.text))
            .map(|(index, candidate)| {
                let (truncated, _) = candidate.unicode_truncate(line_len);
                if index == state.selection_idx() {
                    format!(
                        "{}{}> {}{}",
                        clear::CurrentLine,
                        invert,
                        truncated,
                        no_invert
                    )
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

impl<W: io::Write + Send + Unpin + 'static> Drop for Layout<W> {
    fn drop(&mut self) {
        task::block_on(async {
            if let Some(teardown) = self.screen.teardown() {
                self.write(&teardown)
                    .await
                    .expect("Error writing to output");
            }
        });
    }
}
