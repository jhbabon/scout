//! User Interface rendering logic and components

mod components;
mod convert;

use components::*;

use crate::common::Result;
use crate::config::Config;
use crate::state::{State, StateUpdate};
use ansi_term::Style;
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use std::fmt;
use termion::{clear, cursor};

const ALTERNATE_SCREEN: &str = csi!("?1049h");
const MAIN_SCREEN: &str = csi!("?1049l");

#[derive(Debug, Clone)]
enum Tile {
    Empty,
    Filled { grapheme: char, style: Style },
}

#[derive(Debug)]
struct Canvas {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
    cursor: (usize, usize),
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        let tiles = vec![vec![Tile::Empty; width]; height];
        let cursor = (0, 0);
        Self {
            tiles,
            width,
            height,
            cursor,
        }
    }

    pub fn width(self) -> usize {
        return self.width;
    }

    pub fn height(self) -> usize {
        return self.height;
    }

    pub fn draw_at(
        &mut self,
        row: usize,
        column: usize,
        grapheme: char,
        style: Style,
    ) -> Result<()> {
        // TODO: verify coordinates
        let tile = Tile::Filled { grapheme, style };
        self.tiles[row][column] = tile;

        Ok(())
    }

    pub fn empty_at(&mut self, row: usize, column: usize) -> Result<()> {
        // TODO: verify coordinates
        let tile = Tile::Empty;
        self.tiles[row][column] = tile;

        Ok(())
    }

    pub fn cursor_at(&mut self, row: usize, column: usize) -> Result<()> {
        // TODO: Error if out of boundaries
        self.cursor = (row, column);

        Ok(())
    }
}

impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", cursor::Hide)?;

        for (row, columns) in self.tiles.iter().enumerate() {
            for (column, tile) in columns.iter().enumerate() {
                match tile {
                    Tile::Filled { grapheme, style } => {
                        write!(
                            f,
                            "{}{}",
                            cursor::Goto(column as u16 + 1, row as u16 + 1),
                            style.paint(String::from(*grapheme))
                        )?;
                    }
                    _ => {
                        write!(
                            f,
                            "{}{}",
                            cursor::Goto(column as u16 + 1, row as u16 + 1),
                            Style::default().paint(" ")
                        )?;
                    }
                }
            }
        }

        let (r, c) = self.cursor;
        write!(
            f,
            "{}{}",
            cursor::Goto(c as u16 + 1, r as u16 + 1),
            cursor::Show
        )?;

        Ok(())
    }
}

#[derive(Debug)]
struct Brush<'c> {
    canvas: &'c mut Canvas,
    current_row: usize,
    current_column: usize,
}

impl<'c> Brush<'c> {
    fn new(canvas: &'c mut Canvas) -> Self {
        Self {
            canvas,
            current_row: 0,
            current_column: 0,
        }
    }

    pub fn draw(&mut self, grapheme: char, style: Style) -> Result<()> {
        // TODO: verify position
        self.canvas
            .draw_at(self.current_row, self.current_column, grapheme, style)?;

        // TODO: Move to next row if out of boundaries
        self.current_column += 1;

        Ok(())
    }

    pub fn empty(&mut self) -> Result<()> {
        // TODO: verify position
        self.canvas
            .empty_at(self.current_row, self.current_column)?;

        // TODO: Move to next row if out of boundaries
        self.current_column += 1;

        Ok(())
    }

    pub fn set_cursor(&mut self) -> Result<()> {
        self.canvas.cursor_at(self.current_row, self.current_column)
    }

    pub fn up(&mut self) -> Result<()> {
        // TODO: error if out of boundaries (?)
        if self.current_row > 0 {
            self.current_row -= 1;
        }
        Ok(())
    }

    pub fn down(&mut self) -> Result<()> {
        // TODO: error if out of boundaries (?)
        self.current_row += 1;
        Ok(())
    }

    pub fn go_to(&mut self, row: usize, column: usize) -> Result<()> {
        // TODO: verify coordinates
        self.current_row = row;
        self.current_column = column;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.current_row = 0;
        self.current_column = 0;
    }
}

#[derive(Debug, Clone)]
enum Mode {
    Full,
    Inline(usize),
}

impl Mode {
    // Depending on the mode (full or inline) we want to setup the screen in different ways:
    //
    // * In full screen we want to go to an "Alternate screen". Basically the terminal changes to
    //   another clean "window".
    // * In inline mode we want to make enough room to be able to print lines under the cursor
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

    // After finishing with the program we want to restore the screen
    //
    // * In full mode that means going back to the main screen, with no changes
    // * In inline mode that means cleaning the last line to print the result
    pub fn teardown(&self) -> Option<String> {
        let teardown = match self {
            Self::Full => MAIN_SCREEN.to_string(),
            Self::Inline(_) => format!("{}{}{}", clear::CurrentLine, clear::AfterCursor, "\r"),
        };

        Some(teardown)
    }
}

impl From<&Config> for Mode {
    fn from(config: &Config) -> Self {
        if config.screen.is_full() {
            Self::Full
        } else {
            let height = config.screen.height();
            Self::Inline(height)
        }
    }
}

/// This type represents the screen and how to draw each UI element on it
#[derive(Debug)]
pub struct Painter<W: io::Write + Send + Unpin + 'static> {
    mode: Mode,
    writer: W,
    prompt: PromptComponent,
    gauge: GaugeComponent,
    list: ListComponent,
}

impl<W: io::Write + Send + Unpin + 'static> Painter<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let mode = config.into();
        let prompt = config.into();
        let gauge = config.into();
        let list = config.into();

        let mut canvas = Self {
            mode,
            writer,
            prompt,
            gauge,
            list,
        };

        if let Some(setup) = canvas.mode.setup() {
            canvas.write(&setup).await?;
        }

        Ok(canvas)
    }

    /// Update the UI with the given State
    ///
    /// Printing to the terminal is quite expensive, so the whole system tries to reduce
    /// the number of prints and allocates a few Strings as possible
    pub async fn render(&mut self, state: &State) -> Result<()> {
        match state.last_update() {
            StateUpdate::Query => {
                let mut canvas = Canvas::new(50, 50);
                let mut brush = Brush::new(&mut canvas);
                // TODO: Maybe use `std::io::Cursor` instead of String?
                // let display = format!("{}\r{}", clear::CurrentLine, self.prompt.render(state));
                let mut count = 0;
                for ch in state.query().chars() {
                    brush.draw(ch, Style::default())?;
                    brush.set_cursor()?;
                    count += 1;
                }
                if count < 50 {
                    let left = 50 - count;
                    for _ in std::iter::repeat(()).take(left) {
                        brush.empty()?;
                    }
                }
                let display = format!("{}", canvas);
                self.write(&display).await?;
            }
            _ => {
                // self.list.scroll(state);

                // let list_renderer = self.list.render(state);
                // let list_len = list_renderer.len();

                // // Only add a new line if we are going to print items
                // let gauge_separator = if list_len == 0 { "" } else { "\n" };

                // let display = format!(
                //     "{down}{clrl}\r{gauge}{gauge_sep}{list}{clra}{up}{clrl}\r{prompt}",
                //     clrl = clear::CurrentLine,
                //     down = cursor::Down(1),
                //     gauge = self.gauge.render(state),
                //     gauge_sep = gauge_separator,
                //     list = list_renderer,
                //     clra = clear::AfterCursor,
                //     // By going up and printing as the last element the prompt we ensure the cursor
                //     // is in the right position
                //     up = cursor::Up((list_len + 1) as u16),
                //     prompt = self.prompt.render(state),
                // );

                // self.write(&display).await?;
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

impl<W: io::Write + Send + Unpin + 'static> Drop for Painter<W> {
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
