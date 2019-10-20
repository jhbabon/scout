// use log::debug;
use std::fmt::{self, Write};
use async_std::prelude::*;
use async_std::io;
use termion::terminal_size;
use sublime_fuzzy::format_simple;
use termion;
use crate::result::Result;
use crate::state::State;

#[derive(Debug,Clone)]
pub struct Layout {
    display: Option<String>,
    size: (usize, usize),
    offset: usize,
}

// FIXME: Make layout rendering more efficient
//  * Maybe use a buffwriter in the renderer?
//  * No writeln! macros
//  * Only redraw lines that need change?
//  * Use ansi_term for colors, etc
impl Layout {
    pub fn new() -> Self {
        // TODO: Pass width and height as args or in a Config
        let (width, height) = terminal_size().expect("Error getting terminal size");
        // debug!("Size is {:?}", size);
        let display = None;
        let size = (width as usize, height as usize);
        let offset = 0;

        Self { display, size, offset }
    }

    pub fn update(&mut self, state: &State) -> Result<()> {
        let mut display = String::new();

        // list
        let (offset, lines) = self.scroll(&state);
        let list: Vec<String> = state.matches
            .iter()
            .cloned()
            .enumerate()
            .skip(offset)
            .take(lines)
            .map(|(idx, c)| {
                if let Some(score_match) = c.score_match {
                    (idx, format_simple(&score_match, &c.string, "", ""))
                } else {
                    (idx, c.string)
                }
            })
            .map(|(index, candidate)| {
                let mut selected = " ";
                if index == state.selection_idx() {
                    selected = ">";
                }

                format!("{} {}", selected, candidate)
            })
            .collect();

        write!(&mut display, "\n{}", list.join("\n"))?;

        // prompt
        let prompt = format!("{:width$} > {}", state.matches.len(), state.query_string(), width = 3);
        write!(&mut display, "{}{}", termion::cursor::Goto(1, 1), prompt)?;

        self.display = Some(display);

        Ok(())
    }

    fn scroll(&mut self, state: &State) -> (usize, usize) {
        let (_, height) = self.size;
        let lines_len = height - 1;

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

pub struct Renderer<W> {
    writer: W,
}

impl<W: io::Write + Unpin> Renderer<W> {
    pub fn new(writer: W) -> Self {
        // Looks like we can use the normal terminal size
        // even with ptty
        // debug!("Size is {:?}", );

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
        write!(&mut screen, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
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
