// use log::debug;
use async_std::prelude::*;
use async_std::io;
// use termion::terminal_size;
use crate::result::Result;

pub struct Output<W> {
    writer: W,
}

impl<W: io::Write + Unpin> Output<W> {
    pub fn new(writer: W) -> Self {
        // Looks like we can use the normal terminal size
        // even with ptty
        // debug!("Size is {:?}", terminal_size().unwrap());

        Self { writer }
    }

    pub async fn setup(&mut self) -> Result<()> {
        let alternate = format!("{}", "\x1B[?1049h");

        self.render(alternate).await?;

        Ok(())
    }

    pub async fn teardown(&mut self) -> Result<()> {
        let main = format!("{}", "\x1B[?1049l");

        self.render(main).await?;

        Ok(())
    }

    pub async fn render(&mut self, string: String) -> Result<()> {
        self.writer.write_all(string.as_bytes()).await?;
        self.writer.flush().await?;

        Ok(())
    }
}
