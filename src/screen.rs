// use log::debug;
use crate::common::Result;
use crate::config::Config;
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use termion::{clear, cursor};

const ALTERNATE_SCREEN: &'static str = csi!("?1049h");
const MAIN_SCREEN: &'static str = csi!("?1049l");

enum ScreenKind {
    Full,
    Inline(usize),
}

impl ScreenKind {
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

pub struct Screen<W: io::Write + Send + Unpin + 'static> {
    writer: W,
    kind: ScreenKind,
}

impl<W: io::Write + Send + Unpin + 'static> Screen<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let kind = if config.screen.full {
            ScreenKind::Full
        } else {
            let (_, height) = config.screen.size;
            ScreenKind::Inline(height)
        };

        let mut screen = Self { writer, kind };

        if let Some(setup) = screen.kind.setup() {
            screen.render(&setup).await?;
        }

        Ok(screen)
    }

    pub async fn render<L: std::fmt::Display>(&mut self, layout: &L) -> Result<()> {
        let rendered = format!(
            "{}{}{}",
            termion::cursor::Hide,
            layout,
            termion::cursor::Show,
        );
        self.writer.write_all(rendered.as_bytes()).await?;
        self.writer.flush().await?;

        Ok(())
    }
}

impl<W: io::Write + Send + Unpin + 'static> Drop for Screen<W> {
    fn drop(&mut self) {
        task::block_on(async {
            if let Some(teardown) = self.kind.teardown() {
                self.render(&teardown)
                    .await
                    .expect("Error writing to output");
            }
        });
    }
}
