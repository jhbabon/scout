use regex::Regex;
use rayon::prelude::*;

use choice::Choice;
use pattern::Pattern;
use refine::refine;
use errors::Error;

/// This struct does the fuzzy search over a list of strings
///
/// You create a struct instance with all the list items and then you use that instance to filter
/// the list with different queries (list of chars).
///
/// # Example
///
/// ```
/// use scout::Scout;
///
/// let list = vec!["d/e/f.rs", "a/a/b/c.rs", "a/b/c.rs"];
/// let scout = Scout::new(list);
///
/// let query = ['a', 'b', 'c'];
/// let choices = scout.explore(&query);
///
/// let expected = vec!["a/b/c.rs", "a/a/b/c.rs"];
/// let actual: Vec<String> = choices.into_iter().map(|choice| choice.to_string()).collect();
///
/// assert_eq!(expected, actual);
/// ```
pub struct Scout<'a> {
    list: Vec<&'a str>,
}

impl<'a> Scout<'a> {
    /// Create a new Scout instance with a list of strings
    pub fn new(list: Vec<&'a str>) -> Self {
        Self { list }
    }

    /// Search for the choices that match a query, sorted by best match first.
    ///
    /// If the query is empty, it returns all the choices with the original order of the items.
    pub fn explore<'b>(&self, query: &'b [char]) -> Vec<Choice> {
        if query.is_empty() {
            return self.list
                .iter()
                .map(|text| text.to_string().into())
                .collect::<Vec<Choice>>();
        }

        let re = match self.regex(query) {
            Ok(r) => r,
            Err(e) => panic!("{:?}", e),
        };

        let mut choices: Vec<Choice> = self.list
            .par_iter()
            .map(|line| refine(&re, &line))
            .filter(|choice| choice.is_some())
            .map(|choice| choice.unwrap())
            .collect();

        choices.sort();

        choices
    }

    /// Get a Regex from a list of chars.
    fn regex<'b>(&self, query: &'b [char]) -> Result<Regex, Error> {
        let pattern: Pattern = query.into();
        let regex = Regex::new(&pattern.to_string())?;

        Ok(regex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: [&'static str; 7] = [
        "/some/deeper/path/users.rs",
        "/some/path/api_user.rs",
        "/some/path/user_group.rs",
        "/some/path/use_remote.rs",
        "foobar.rs",
        "reserved?*.rs",
        "ßℝ💣",
    ];

    #[test]
    fn it_gets_best_matches() {
        let query = ['u', 's', 'r'];
        let expected = vec![
            Choice::new("/some/path/user_group.rs".to_string(), 11, 15),
            Choice::new("/some/path/api_user.rs".to_string(), 15, 19),
            Choice::new("/some/deeper/path/users.rs".to_string(), 18, 22),
            Choice::new("/some/path/use_remote.rs".to_string(), 11, 16),
        ];

        let scout = Scout::new(LIST.to_vec());

        assert_eq!(expected, scout.explore(&query));
    }

    #[test]
    fn it_is_case_insensitive() {
        let query = ['U', 's', 'R'];
        let expected = vec![
            Choice::new("/some/path/user_group.rs".to_string(), 11, 15),
            Choice::new("/some/path/api_user.rs".to_string(), 15, 19),
            Choice::new("/some/deeper/path/users.rs".to_string(), 18, 22),
            Choice::new("/some/path/use_remote.rs".to_string(), 11, 16),
        ];

        let scout = Scout::new(LIST.to_vec());

        assert_eq!(expected, scout.explore(&query));
    }

    #[test]
    fn it_takes_reserved_chars() {
        let query = ['?', '*', '.'];
        let expected = vec![Choice::new("reserved?*.rs".to_string(), 8, 11)];

        let scout = Scout::new(LIST.to_vec());

        assert_eq!(expected, scout.explore(&query));
    }

    #[test]
    fn it_takes_special_chars() {
        let query = ['ß', '💣'];
        let expected = vec![Choice::new("ßℝ💣".to_string(), 0, 9)];

        let scout = Scout::new(LIST.to_vec());

        assert_eq!(expected, scout.explore(&query));
    }

    #[test]
    fn it_returns_the_same_on_empty_query() {
        let query = [];
        let expected: Vec<String> = LIST.iter().map(|&s| String::from(s)).collect();

        let scout = Scout::new(LIST.to_vec());

        let choices: Vec<String> = scout
            .explore(&query)
            .iter()
            .map(|choice| choice.to_string())
            .collect();

        assert_eq!(expected, choices);
    }
}
