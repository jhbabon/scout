use crate::common::Result;
use crate::config::Config;
use ansi_term::Style;
use std::fmt;
use termion::cursor;

#[derive(Debug, Clone)]
enum Tile {
    Empty,
    Filled { grapheme: String, style: Style },
}

#[derive(Debug)]
pub struct Canvas {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
    cursor: (usize, usize),
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        // Instead fo a matrix lets use a unique vector
        let tiles = vec![Tile::Empty; width * height];
        let cursor = (0, 0);
        Self {
            tiles,
            width,
            height,
            cursor,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn draw_at(
        &mut self,
        row: usize,
        column: usize,
        grapheme: String,
        style: Style,
    ) -> Result<()> {
        // TODO: verify coordinates
        let tile = Tile::Filled { grapheme, style };
        self.tiles[row * self.width + column] = tile;

        Ok(())
    }

    pub fn empty_at(&mut self, row: usize, column: usize) -> Result<()> {
        // TODO: verify coordinates
        let tile = Tile::Empty;
        self.tiles[row * self.width + column] = tile;

        Ok(())
    }

    pub fn cursor_at(&mut self, row: usize, column: usize) -> Result<()> {
        // TODO: Error if out of boundaries
        self.cursor = (row, column);

        Ok(())
    }
}

impl From<&Config> for Canvas {
    fn from(config: &Config) -> Self {
        let height = config.screen.height();
        let width = config.screen.width();

        Self::new(width, height)
    }
}

impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", cursor::Hide)?;

        for (position, tile) in self.tiles.iter().enumerate() {
            let column = position % self.width;
            let row = (position - column) / self.width;
            match tile {
                Tile::Filled { grapheme, style } => {
                    write!(
                        f,
                        "{}{}",
                        cursor::Goto(column as u16 + 1, row as u16 + 1),
                        style.paint(grapheme)
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
pub struct Projector {
    origin: (usize, usize),
    width: usize,
    height: usize,
}

impl Projector {
    pub fn new(origin: (usize, usize), width: usize, height: usize) -> Self {
        Self {
            origin,
            width,
            height,
        }
    }

    pub fn project_row(&self, relative: usize) -> usize {
        let (_, y) = self.origin;

        relative + y
    }

    pub fn project_column(&self, relative: usize) -> usize {
        let (x, _) = self.origin;

        relative + x
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug)]
pub struct Brush<'c> {
    canvas: &'c mut Canvas,
    projector: &'c Projector,
    current_row: usize,
    current_column: usize,
}

impl<'c> Brush<'c> {
    pub fn new(canvas: &'c mut Canvas, projector: &'c Projector) -> Self {
        Self {
            canvas,
            projector,
            current_row: 0,
            current_column: 0,
        }
    }

    pub fn width(&self) -> usize {
        self.projector.width()
    }

    pub fn height(&self) -> usize {
        self.projector.height()
    }

    fn projected_row(&self) -> usize {
        self.projector.project_row(self.current_row)
    }

    fn projected_column(&self) -> usize {
        self.projector.project_column(self.current_column)
    }

    pub fn clear_until_eol(&mut self) -> Result<()> {
        while self.current_column < self.last_column() {
            self.empty()?;
        }
        self.empty()?; // clear last column

        Ok(())
    }

    pub fn clear_until_eof(&mut self) -> Result<()> {
        while self.current_row < self.last_row() {
            self.clear_until_eol()?;
            self.new_line()?;
        }
        self.clear_until_eol()?; // clear last row

        Ok(())
    }

    fn last_row(&self) -> usize {
        self.height() - 1
    }

    fn last_column(&self) -> usize {
        self.width() - 1
    }

    pub fn draw(&mut self, grapheme: String, style: Style) -> Result<()> {
        // TODO: verify position
        self.canvas.draw_at(
            self.projected_row(),
            self.projected_column(),
            grapheme,
            style,
        )?;

        // TODO: Move to next row if out of boundaries
        self.right()?;

        Ok(())
    }

    pub fn empty(&mut self) -> Result<()> {
        // TODO: verify position
        self.canvas
            .empty_at(self.projected_row(), self.projected_column())?;

        // TODO: Move to next row if out of boundaries
        self.right()?;

        Ok(())
    }

    pub fn set_cursor(&mut self) -> Result<()> {
        self.canvas
            .cursor_at(self.projected_row(), self.projected_column())
    }

    pub fn left(&mut self) -> Result<()> {
        // TODO: error if out of boundaries (?)
        if self.current_column > 0 {
            self.current_column -= 1;
        }
        Ok(())
    }

    pub fn right(&mut self) -> Result<()> {
        // TODO: error if out of boundaries (?)
        if self.current_column < self.last_column() {
            self.current_column += 1;
        }
        Ok(())
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
        if self.current_row < self.last_row() {
            self.current_row += 1;
        }
        Ok(())
    }

    pub fn go_to(&mut self, row: usize, column: usize) -> Result<()> {
        // TODO: verify coordinates
        self.current_row = row;
        self.current_column = column;

        Ok(())
    }

    pub fn new_line(&mut self) -> Result<()> {
        self.down()?;
        self.current_column = 0;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.current_row = 0;
        self.current_column = 0;
    }
}
