use std::fmt;
use std::cmp::Ordering;

use super::score::Score;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct OrderlyStr<'a>(&'a str);

impl<'a> Ord for OrderlyStr<'a> {
    fn cmp(&self, other: &OrderlyStr) -> Ordering {
        self.0.len().cmp(&other.0.len())
    }
}

impl<'a> PartialOrd for OrderlyStr<'a> {
    fn partial_cmp(&self, other: &OrderlyStr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> fmt::Display for OrderlyStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a str> for OrderlyStr<'a> {
    fn from(text: &'a str) -> Self {
        OrderlyStr(text)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice<'a> {
    match_start: usize,
    match_end: usize,
    score: Score,
    text: OrderlyStr<'a>,
}

impl<'a> Choice<'a> {
    pub fn new(text: &'a str, match_start: usize, match_end: usize) -> Choice<'a> {
        Choice {
            match_start,
            match_end,
            score: (match_start, match_end).into(),
            text: text.into(),
        }
    }

    pub fn start(&self) -> usize {
        self.match_start
    }

    pub fn end(&self) -> usize {
        self.match_end
    }
}

impl<'a> Ord for Choice<'a> {
    fn cmp(&self, other: &Choice) -> Ordering {
        let by_score = self.score.cmp(&other.score);

        by_score.then_with(|| self.text.cmp(&other.text))
    }
}

impl<'a> PartialOrd for Choice<'a> {
    fn partial_cmp(&self, other: &Choice) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> fmt::Display for Choice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl<'a> From<(&'a str, usize, usize)> for Choice<'a> {
    fn from(tuple: (&'a str, usize, usize)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

impl<'a> From<&'a str> for Choice<'a> {
    fn from(text: &'a str) -> Self {
        Self::new(text, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_a_new_choice() {
        let text = "abcde";
        let expected = Choice {
            match_start: 1,
            match_end: 3,
            score: (1, 3).into(),
            text: text.into(),
        };

        assert_eq!(expected, Choice::new(text, 1, 3));
    }

    #[test]
    fn it_builds_a_new_choice_from_a_tuple() {
        let text = "abcde";
        let expected = Choice {
            match_start: 1,
            match_end: 3,
            score: (1, 3).into(),
            text: text.into(),
        };

        assert_eq!(expected, (text, 1, 3).into());
    }

    #[test]
    fn it_builds_a_new_choice_from_a_str() {
        let text = "abcde";
        let expected = Choice {
            match_start: 0,
            match_end: 0,
            score: (0, 0).into(),
            text: text.into(),
        };

        assert_eq!(expected, text.into());
    }

    #[test]
    fn it_orders_first_choices_with_better_matching() {
        let text = "aacde";
        let better = Choice::new(text, 1, 3);
        let worse = Choice::new(text, 0, 3);

        assert!(better < worse);
    }

    #[test]
    fn on_equal_scores_it_orders_first_choices_with_shorter_text() {
        let better = Choice::new("bbb", 0, 3);
        let worse = Choice::new("abbb", 0, 3);

        assert!(better < worse);
    }
}
