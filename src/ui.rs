mod components;
mod convert;

use components::*;

use crate::common::Result;
use crate::config::Config;
use crate::state::{State, StateUpdate};
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use termion::{clear, cursor};

const ALTERNATE_SCREEN: &'static str = csi!("?1049h");
const MAIN_SCREEN: &'static str = csi!("?1049l");

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

#[derive(Debug)]
pub struct Canvas<W: io::Write + Send + Unpin + 'static> {
    mode: Mode,
    writer: W,
    prompt: PromptComponent,
    gauge: GaugeComponent,
    list: ListComponent,
}

impl<W: io::Write + Send + Unpin + 'static> Canvas<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let mode = if config.screen.is_full() {
            Mode::Full
        } else {
            let height = config.screen.height();
            Mode::Inline(height)
        };

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

    pub async fn render(&mut self, state: &State) -> Result<()> {
        match state.last_update() {
            StateUpdate::Query => {
                let display = format!("{}\r{}", clear::CurrentLine, self.prompt.render(state));
                self.write(&display).await?;
            }
            _ => {
                self.list.scroll(state);

                let list_renderer = self.list.render(state);
                let list_len = list_renderer.len() as u16;

                // Only add a new line if we are going to print items
                let gauge_separator = if list_len == 0 { "" } else { "\n" };

                let display = format!(
                    "{}{}\r{}{}{}{}{}{}\r{}",
                    cursor::Down(1),
                    clear::CurrentLine,
                    self.gauge.render(state),
                    gauge_separator,
                    list_renderer,
                    clear::AfterCursor,
                    // We always need to reprint the prompt after
                    // going up to set the cursor in the last
                    // position
                    cursor::Up(list_len + 1),
                    clear::CurrentLine,
                    self.prompt.render(state),
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

impl<W: io::Write + Send + Unpin + 'static> Drop for Canvas<W> {
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
