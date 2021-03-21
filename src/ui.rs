//! User Interface rendering logic and components

// FIXME: The new render system based on a matrix doesn't work on inline mode. It uses absolute
// coordinates and in inline mode these are relative to the position where the command was invoked
//
// In order to get the inline coordinates the program has to print the sequence "\x1B[6n" to the current
// pttyout and wait to read the bytes printed to pttyin
// see https://gitlab.redox-os.org/redox-os/termion/-/blob/master/src/cursor.rs#L139-183


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

        let mut painter = Self {
            mode,
            canvas,
            writer,
            prompt,
            gauge,
            list,
        };

        if let Some(setup) = painter.mode.setup() {
            painter.write(&setup).await?;
        }

        Ok(painter)
    }

    /// Update the UI with the given State
    ///
    /// Printing to the terminal is quite expensive, so the whole system tries to reduce
    /// the number of prints and allocates a few Strings as possible
    pub async fn render(&mut self, state: &State) -> Result<()> {
        log::debug!("{:?}", state.last_update());
        match state.last_update() {
            StateUpdate::Init => {
                // clean the screen
                self.write_canvas().await?;

                self.prompt.render(state, &mut self.canvas)?;
                self.gauge.render(state, &mut self.canvas)?;

                // TODO: Move scroll inside #draw
                self.list.scroll(state);
                self.list.render(state, &mut self.canvas)?;

                self.write_canvas().await?;
            }
            StateUpdate::Query => {
                self.prompt.render(state, &mut self.canvas)?;
                // TODO: Maybe use `std::io::Cursor` instead of String?
                // let display = format!("{}\r{}", clear::CurrentLine, self.prompt.render(state));
                self.write_canvas().await?;
            }
            // TODO: more fine grained status
            _ => {
                // self.prompt.render(state, &mut self.canvas)?;
                self.gauge.render(state, &mut self.canvas)?;

                // TODO: Move scroll inside #draw
                self.list.scroll(state);
                self.list.render(state, &mut self.canvas)?;

                self.write_canvas().await?;
            }
        }

        Ok(())
    }

    async fn write_canvas(&mut self) -> Result<()> {
        let display = format!("{}", self.canvas);
        self.write(&display).await?;

        self.canvas.flush();

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
