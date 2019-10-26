// use log::debug;
use async_std::io;
use async_std::task;
use async_std::prelude::*;
use termion::{cursor,clear};
use crate::config::Config;
use crate::common::Result;

enum ScreenKind {
    Full,
    Inline,
}

impl ScreenKind {
    pub fn setup(&self) -> Option<String> {
        let setup = match self {
            Self::Full => format!("{}{}", csi!("?1049h"), cursor::Goto(1,1)),
            // FIXME: This doesn't work if the screen is full
            Self::Inline => "\r".to_string(),
        };

        Some(setup)
    }

    pub fn teardown(&self) -> Option<String> {
        let teardown = match self {
            Self::Full => csi!("?1049l").to_string(),
            Self::Inline => format!(
                "{}{}{}",
                clear::CurrentLine,
                clear::AfterCursor,
                "\r\n"
            ),
        };

        Some(teardown)
    }
}

pub struct Screen<W: io::Write + Send + Unpin + 'static> {
    writer: W,
    kind: ScreenKind,
}

// TODO: Make configurable
impl<W: io::Write + Send + Unpin + 'static> Screen<W> {
    pub async fn new(config: &Config, writer: W) -> Result<Self> {
        let kind = if config.screen.full {
            ScreenKind::Full
        } else {
            ScreenKind::Inline
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
                self.render(&teardown).await
                    .expect("Error writing to output");
            }
        });
    }
}
