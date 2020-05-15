use core::convert::Infallible;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fmt;
use std::iter::IntoIterator;
use std::str::FromStr;

#[derive(Debug)]
pub struct ParseColorError;

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing color")
    }
}

impl Error for ParseColorError {}

#[derive(Debug)]
pub struct ParseRuleError;

impl fmt::Display for ParseRuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing color")
    }
}

impl Error for ParseRuleError {}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Black,
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
    Cyan,
    White,
    Fixed(u8),
    RGB(u8, u8, u8),
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('#') {
            let r: u8 = u8::from_str_radix(&s[1..3], 16).map_err(|_| ParseColorError)?;
            let g: u8 = u8::from_str_radix(&s[3..5], 16).map_err(|_| ParseColorError)?;
            let b: u8 = u8::from_str_radix(&s[5..7], 16).map_err(|_| ParseColorError)?;

            return Ok(Color::RGB(r, g, b));
        }

        match s {
            "black" => Ok(Self::Black),
            "red" => Ok(Self::Red),
            "yellow" => Ok(Self::Yellow),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "purple" => Ok(Self::Purple),
            "cyan" => Ok(Self::Cyan),
            "white" => Ok(Self::White),
            "bright-black" => Ok(Self::Fixed(8)),
            "bright-red" => Ok(Self::Fixed(9)),
            "bright-green" => Ok(Self::Fixed(10)),
            "bright-yellow" => Ok(Self::Fixed(11)),
            "bright-blue" => Ok(Self::Fixed(12)),
            "bright-purple" => Ok(Self::Fixed(13)),
            "bright-cyan" => Ok(Self::Fixed(14)),
            "bright-white" => Ok(Self::Fixed(15)),
            maybe_fixed => maybe_fixed
                .parse::<u8>()
                .map(|f| Self::Fixed(f))
                .map_err(|_| ParseColorError),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Rule {
    Reset,
    Underline,
    Strikethrough,
    Reverse,
    Bold,
    Italic,
    Dimmed,
    Fg(Color),
    Bg(Color),
}

impl FromStr for Rule {
    type Err = ParseRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("fg:") {
            return s
                .trim_start_matches("fg:")
                .parse()
                .map(|c| Self::Fg(c))
                .map_err(|_| ParseRuleError);
        };

        if s.starts_with("bg:") {
            return s
                .trim_start_matches("bg:")
                .parse()
                .map(|c| Self::Bg(c))
                .map_err(|_| ParseRuleError);
        };

        match s {
            "none" => Ok(Self::Reset),
            "underline" => Ok(Self::Underline),
            "strikethrough" => Ok(Self::Strikethrough),
            "reverse" => Ok(Self::Reverse),
            "bold" => Ok(Self::Bold),
            "italic" => Ok(Self::Italic),
            "dimmed" => Ok(Self::Dimmed),
            _ => Err(ParseRuleError),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    rules: Vec<Rule>,
}

impl Style {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }
}

impl FromStr for Style {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rules: Vec<Rule> = vec![];

        let mut iter = s.split_whitespace();

        while let Some(s) = iter.next() {
            match s.parse() {
                Ok(Rule::Reset) => {
                    rules = vec![Rule::Reset];
                    break;
                }
                Ok(rule) => rules.push(rule),
                Err(_) => {}
            }
        }

        Ok(Self::new(rules))
    }
}

impl IntoIterator for Style {
    type Item = Rule;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}

struct StyleVisitor;

impl<'de> Visitor<'de> for StyleVisitor {
    type Value = Style;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a style string. i.e: 'underline bold fg:blue bg:white'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Style {
    fn deserialize<D>(deserializer: D) -> Result<Style, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StyleVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    fn assert_style_from_str(string: &str, rules: Vec<Rule>) {
        let expected = Style::new(rules);
        let actual = Style::from_str(string);

        assert_eq!(actual, Ok(expected));
    }

    #[test]
    fn style_from_str_with_one_rule_test() {
        assert_style_from_str("underline", vec![Rule::Underline]);
        assert_style_from_str("strikethrough", vec![Rule::Strikethrough]);
        assert_style_from_str("reverse", vec![Rule::Reverse]);
        assert_style_from_str("bold", vec![Rule::Bold]);
        assert_style_from_str("italic", vec![Rule::Italic]);
        assert_style_from_str("dimmed", vec![Rule::Dimmed]);
    }

    #[test]
    fn style_from_str_with_many_rules_test() {
        assert_style_from_str(
            "underline strikethrough reverse",
            vec![Rule::Underline, Rule::Strikethrough, Rule::Reverse],
        );
        assert_style_from_str(
            "bold italic dimmed",
            vec![Rule::Bold, Rule::Italic, Rule::Dimmed],
        );
    }

    #[test]
    fn style_from_str_with_none_rule_test() {
        assert_style_from_str("none", vec![Rule::Reset]);
        assert_style_from_str("underline none bold fb:blue", vec![Rule::Reset]);
    }

    #[test]
    fn style_from_str_with_foreground_color_test() {
        assert_style_from_str("fg:black", vec![Rule::Fg(Color::Black)]);
        assert_style_from_str("fg:red", vec![Rule::Fg(Color::Red)]);
        assert_style_from_str("fg:yellow", vec![Rule::Fg(Color::Yellow)]);
        assert_style_from_str("fg:green", vec![Rule::Fg(Color::Green)]);
        assert_style_from_str("fg:blue", vec![Rule::Fg(Color::Blue)]);
        assert_style_from_str("fg:purple", vec![Rule::Fg(Color::Purple)]);
        assert_style_from_str("fg:cyan", vec![Rule::Fg(Color::Cyan)]);
        assert_style_from_str("fg:white", vec![Rule::Fg(Color::White)]);
    }

    #[test]
    fn style_from_str_with_bright_foreground_color_test() {
        assert_style_from_str("fg:bright-black", vec![Rule::Fg(Color::Fixed(8))]);
        assert_style_from_str("fg:bright-red", vec![Rule::Fg(Color::Fixed(9))]);
        assert_style_from_str("fg:bright-green", vec![Rule::Fg(Color::Fixed(10))]);
        assert_style_from_str("fg:bright-yellow", vec![Rule::Fg(Color::Fixed(11))]);
        assert_style_from_str("fg:bright-blue", vec![Rule::Fg(Color::Fixed(12))]);
        assert_style_from_str("fg:bright-purple", vec![Rule::Fg(Color::Fixed(13))]);
        assert_style_from_str("fg:bright-cyan", vec![Rule::Fg(Color::Fixed(14))]);
        assert_style_from_str("fg:bright-white", vec![Rule::Fg(Color::Fixed(15))]);
    }

    #[test]
    fn style_from_str_with_background_color_test() {
        assert_style_from_str("bg:black", vec![Rule::Bg(Color::Black)]);
        assert_style_from_str("bg:red", vec![Rule::Bg(Color::Red)]);
        assert_style_from_str("bg:yellow", vec![Rule::Bg(Color::Yellow)]);
        assert_style_from_str("bg:green", vec![Rule::Bg(Color::Green)]);
        assert_style_from_str("bg:blue", vec![Rule::Bg(Color::Blue)]);
        assert_style_from_str("bg:purple", vec![Rule::Bg(Color::Purple)]);
        assert_style_from_str("bg:cyan", vec![Rule::Bg(Color::Cyan)]);
        assert_style_from_str("bg:white", vec![Rule::Bg(Color::White)]);
    }

    #[test]
    fn style_from_str_with_bright_background_color_test() {
        assert_style_from_str("bg:bright-black", vec![Rule::Bg(Color::Fixed(8))]);
        assert_style_from_str("bg:bright-red", vec![Rule::Bg(Color::Fixed(9))]);
        assert_style_from_str("bg:bright-green", vec![Rule::Bg(Color::Fixed(10))]);
        assert_style_from_str("bg:bright-yellow", vec![Rule::Bg(Color::Fixed(11))]);
        assert_style_from_str("bg:bright-blue", vec![Rule::Bg(Color::Fixed(12))]);
        assert_style_from_str("bg:bright-purple", vec![Rule::Bg(Color::Fixed(13))]);
        assert_style_from_str("bg:bright-cyan", vec![Rule::Bg(Color::Fixed(14))]);
        assert_style_from_str("bg:bright-white", vec![Rule::Bg(Color::Fixed(15))]);
    }

    #[test]
    fn style_from_str_with_fixed_color_test() {
        assert_style_from_str(
            "fg:1 bg:127",
            vec![Rule::Fg(Color::Fixed(1)), Rule::Bg(Color::Fixed(127))],
        );
    }

    #[test]
    fn style_from_str_with_rgb_color_test() {
        assert_style_from_str(
            "fg:#001122 bg:#ffbbcc",
            vec![
                Rule::Fg(Color::RGB(0, 17, 34)),
                Rule::Bg(Color::RGB(255, 187, 204)),
            ],
        );
    }

    #[test]
    fn style_from_str_with_mixed_rules_and_colors_test() {
        assert_style_from_str(
            "underline bold fg:127 bg:black",
            vec![
                Rule::Underline,
                Rule::Bold,
                Rule::Fg(Color::Fixed(127)),
                Rule::Bg(Color::Black),
            ],
        );

        assert_style_from_str(
            "dimmed fg:#ffffff reverse",
            vec![
                Rule::Dimmed,
                Rule::Fg(Color::RGB(255, 255, 255)),
                Rule::Reverse,
            ],
        );
    }

    #[test]
    fn style_deserialization_test() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Subject {
            style: Option<Style>,
        }

        let style = Style::new(vec![Rule::Underline, Rule::Fg(Color::Fixed(22))]);
        let expected = Subject { style: Some(style) };
        let content = r#"
            style = 'underline fg:22'
        "#;
        let actual: Subject = toml::from_str(content).unwrap();
        assert_eq!(actual, expected);

        let expected = Subject { style: None };
        let content = r#"
            nothing = true
        "#;
        let actual: Subject = toml::from_str(content).unwrap();
        assert_eq!(actual, expected);
    }
}
