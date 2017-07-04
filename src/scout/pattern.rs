use regex;
use std::fmt;

/// Get a valid regex pattern from a set of chars (the query).
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pattern(String);

impl Pattern {
    /// Create a new valid Regex pattern from a query of chars
    ///
    /// # Example
    ///
    /// ```rust
    /// use scout::Pattern;
    ///
    /// let query = ['a', 'b', 'c'];
    /// let pattern = Pattern::new(&query);
    ///
    /// assert_eq!("(?i)a.*?b.*?c", pattern.to_string());
    /// ```
    pub fn new(query: &[char]) -> Self {
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

impl<'a> From<&'a [char]> for Pattern {
    fn from(query: &[char]) -> Self {
        Pattern::new(query)
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_a_pattern_from_a_set_of_chars() {
        let query = vec!['a', 'b', 'c'];
        let expected = "(?i)a.*?b.*?c".to_string();
        let actual = Pattern::new(&query);

        assert_eq!(expected, actual.to_string());
    }
}
