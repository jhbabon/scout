extern crate regex;

use std::fmt;
use regex::Regex;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Choice<'a> {
    rank: usize,
    subrank: usize,
    string: &'a str,
}

impl<'b, 'a> Choice<'a> {
    pub fn build(re: &'b Regex, string: &'a str) -> Option<Choice<'a>> {
        match re.find(string) {
            Some(matching) => {
                let choice = Choice {
                    rank: matching.end() - matching.start(),
                    subrank: matching.start(),
                    string: string,
                };

                Some(choice)
            },
            None => None
        }
    }

    pub fn from(string: &'a str) -> Choice<'a> {
        Choice { string: string, ..Default::default() }
    }

    pub fn start(&self) -> usize {
        self.subrank
    }

    pub fn end(&self) -> usize {
        self.rank + self.subrank
    }
}

impl<'a> fmt::Display for Choice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

/// Get the version of the program.
pub fn version() -> String {
    let (maj, min, pat) = (option_env!("CARGO_PKG_VERSION_MAJOR"),
                           option_env!("CARGO_PKG_VERSION_MINOR"),
                           option_env!("CARGO_PKG_VERSION_PATCH"));

    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_string(),
    }
}

// Idea taken from:
//   http://blog.amjith.com/fuzzyfinder-in-10-lines-of-python
//
// TODO: Return Result to handle errors
pub fn explore<'a>(list: &'a [&'a str], query: &'a [char]) -> Vec<Choice<'a>> {
    if query.is_empty() {
        return list.into_iter()
            .map(|string| Choice::from(string))
            .collect::<Vec<Choice>>()
    }

    let pattern = build_pattern(query);
    let re = Regex::new(&pattern).unwrap();

    let mut choices: Vec<Choice> = list.iter()
        .map(|string| Choice::build(&re, string))
        .filter(|choice| choice.is_some())
        .map(|choice| choice.unwrap())
        .collect();

    choices.sort();

    choices
}

fn build_pattern<'a>(query: &'a [char]) -> String {
    query.iter()
        .map(|ch| regex::escape(&ch.to_string()))
        .map(|esc| format!("(?i){}.*?", esc)) // (?i) for case insensitive
        .collect()
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
            Choice { rank: 4, subrank: 11, string: "/some/path/user_group.rs" },
            Choice { rank: 4, subrank: 15, string: "/some/path/api_user.rs" },
            Choice { rank: 4, subrank: 18, string: "/some/deeper/path/users.rs" },
            Choice { rank: 5, subrank: 11, string: "/some/path/use_remote.rs" },
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_is_case_insensitive() {
        let query = ['U', 's', 'R'];
        let expected = vec![
            Choice { rank: 4, subrank: 11, string: "/some/path/user_group.rs" },
            Choice { rank: 4, subrank: 15, string: "/some/path/api_user.rs" },
            Choice { rank: 4, subrank: 18, string: "/some/deeper/path/users.rs" },
            Choice { rank: 5, subrank: 11, string: "/some/path/use_remote.rs" },
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_reserved_chars() {
        let query = ['?', '*', '.'];
        let expected = vec![
            Choice { rank: 3, subrank: 8, string: "reserved?*.rs" }
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_special_chars() {
        let query = ['ß', '💣'];
        let expected = vec![
            Choice { rank: 9, subrank: 0, string: "ßℝ💣" }
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_returns_the_same_on_empty_query() {
        let query = [];
        let list = LIST;
        let expected: Vec<String> = list.iter()
            .map(|&s| String::from(s))
            .collect();

        let choices: Vec<String> = explore(&list, &query).iter()
            .map(|choice| choice.to_string())
            .collect();

        assert_eq!(expected, choices);
    }
}
