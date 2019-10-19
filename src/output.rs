// use log::debug;
use std::fmt::{self, Write};
use async_std::prelude::*;
use async_std::io;
// use termion::terminal_size;
use sublime_fuzzy::format_simple;
use termion;
use crate::result::Result;
use crate::state::State;

#[derive(Debug,Clone)]
pub struct Layout {
    display: Option<String>,
}

impl Layout {
    pub fn new() -> Self {
        Self { display: None }
    }

    pub fn update(&mut self, state: &State) -> Result<()> {
        let mut display = String::new();

        // list
        let list: Vec<String> = state.matches
            .iter()
            .cloned()
            .map(|c| format_simple(&c.score_match, &c.string, "", ""))
            .collect();

        for l in list {
            writeln!(&mut display, "  {}", l)?;
        }

        // prompt
        let prompt = format!("{:width$} > {}", state.matches.len(), state.query_string(), width = 3);
        write!(&mut display, "{}{}", termion::cursor::Goto(1, 1), prompt)?;

        self.display = Some(display);

        Ok(())
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

pub struct Renderer<W> {
    writer: W,
}

impl<W: io::Write + Unpin> Renderer<W> {
    pub fn new(writer: W) -> Self {
        // Looks like we can use the normal terminal size
        // even with ptty
        // debug!("Size is {:?}", terminal_size().unwrap());

        Self { writer }
    }

    pub async fn setup(&mut self) -> Result<()> {
        self.write(&"\x1B[?1049h").await?;

        Ok(())
    }

    pub async fn teardown(&mut self) -> Result<()> {
        self.write(&"\x1B[?1049l").await?;

        Ok(())
    }

    pub async fn render<L: std::fmt::Display>(&mut self, layout: &L) -> Result<()> {
        let mut screen = String::new();
        writeln!(&mut screen, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
        write!(&mut screen, "{}", layout)?;

        self.write(&screen).await?;

        Ok(())
    }

    async fn write(&mut self, string: &str) -> Result<()> {
        self.writer.write_all(string.as_bytes()).await?;
        self.writer.flush().await?;

        Ok(())
    }
}
