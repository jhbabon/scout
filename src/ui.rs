//! User Interface rendering logic and components

mod components;
mod convert;
mod painting;

use components::*;
use painting::*;

use crate::common::Result;
use crate::config::Config;
use crate::state::{State, StateUpdate};
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use termion::{clear, cursor};

const ALTERNATE_SCREEN: &str = csi!("?1049h");
const MAIN_SCREEN: &str = csi!("?1049l");

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
    canvas: Canvas,
    prompt: PromptComponent,
    gauge: GaugeComponent,
    list: ListComponent,
}

impl<W: io::Write + Send + Unpin + 'static> Painter<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let mode = config.into();
        let canvas = config.into();
        let prompt = config.into();
        let gauge = config.into();
        let list = config.into();

        let mut canvas = Self {
            mode,
            canvas,
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
        let mut brush = Brush::new(&mut self.canvas);
        match state.last_update() {
            StateUpdate::Query => {
                self.prompt.draw(state, &mut brush)?;
                let display = format!("{}", self.canvas);
                // TODO: Maybe use `std::io::Cursor` instead of String?
                // let display = format!("{}\r{}", clear::CurrentLine, self.prompt.render(state));
                self.write(&display).await?;
            }
            // TODO: more fine grained status
            _ => {
                self.prompt.draw(state, &mut brush)?;
                brush.new_line()?;
                self.gauge.draw(state, &mut brush)?;

                let display = format!("{}", self.canvas);
                self.write(&display).await?;

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
