use regex;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pattern(String);

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Pattern {
    pub fn build<'a>(query: &'a [char]) -> Self {
        let last_index = query.len() - 1;
        let partial: String = query[0..last_index]
            .iter()
            .map(|ch| regex::escape(&ch.to_string()))
            .map(|esc| format!("{}.*?", esc))
            .collect();

        let pattern = format!(
            "(?i){}{}",
            partial,
            regex::escape(&query[last_index].to_string())
        );

        Pattern(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_a_pattern_from_a_set_of_chars() {
        let query = vec!['a', 'b', 'c'];
        let expected = "(?i)a.*?b.*?c".to_string();
        let actual = Pattern::build(&query);

        assert_eq!(expected, actual.to_string());
    }
}
