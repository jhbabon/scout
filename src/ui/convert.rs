use crate::config::styling;
use ansi_term::{Color, Style};

impl Into<Color> for styling::Color {
    fn into(self) -> Color {
        match self {
            styling::Color::Black => Color::Black,
            styling::Color::Red => Color::Red,
            styling::Color::Yellow => Color::Yellow,
            styling::Color::Green => Color::Green,
            styling::Color::Blue => Color::Blue,
            styling::Color::Purple => Color::Purple,
            styling::Color::Cyan => Color::Cyan,
            styling::Color::White => Color::White,
            styling::Color::Fixed(n) => Color::Fixed(n),
            styling::Color::RGB(r, g, b) => Color::RGB(r, g, b),
        }
    }
}

impl Into<Style> for styling::Style {
    fn into(self) -> Style {
        self.into_iter()
            .fold(Style::default(), |acc, rule| match rule {
                styling::Rule::Reset => Style::default(),
                styling::Rule::Underline => acc.underline(),
                styling::Rule::Strikethrough => acc.strikethrough(),
                styling::Rule::Reverse => acc.reverse(),
                styling::Rule::Bold => acc.bold(),
                styling::Rule::Italic => acc.italic(),
                styling::Rule::Dimmed => acc.dimmed(),
                styling::Rule::Fg(color) => acc.fg(color.into()),
                styling::Rule::Bg(color) => acc.on(color.into()),
            })
    }
}
