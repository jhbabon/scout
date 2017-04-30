extern crate regex;

use std::fmt;
use std::cmp::Ordering;
use regex::Regex;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Choice<'a> {
    rank: usize,
    subrank: usize,
    string: SortableStr<'a>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct SortableStr<'a>(&'a str);

impl<'a> Ord for SortableStr<'a> {
    fn cmp(&self, other: &SortableStr) -> Ordering {
        self.0.len().cmp(&other.0.len())
    }
}

impl<'a> PartialOrd for SortableStr<'a> {
    fn partial_cmp(&self, other: &SortableStr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> fmt::Display for SortableStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> fmt::Display for Choice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl<'b, 'a> Choice<'a> {
    // TODO: Move the ranking logic out of Choice. Maybe a Rank/Score struct?
    pub fn build(re: &'b Regex, string: &'a str) -> Option<Choice<'a>> {
        let mut indexes = string.char_indices().map(|(index, _)| index);
        let mut matches = vec![];
        let mut last_match = 0;

        loop {
            let last = last_match;
            // We don't need to iterate over each index, just the ones
            // after the last match from the regex
            let mut iter = indexes.by_ref().skip_while(|&index| last > index);

            match iter.next() {
                Some(index) => {
                    let ma = re.find(&string[index..]);
                    match ma {
                        Some(matching) => {
                            last_match = matching.start();
                            matches.push((index, matching))
                        },
                        None => break
                    }
                }
                None => break
            }
        }
        let min = matches.into_iter()
            .min_by_key(|&(_, m)| (m.end() - m.start(), m.start()));

        match min {
            Some((offset, matching)) => {
                let choice = Choice::new(
                    matching.end() - matching.start(),
                    matching.start() + offset,
                    string
                );

                Some(choice)
            },
            None => None
        }
    }

    pub fn new(rank: usize, subrank: usize, string: &'a str) -> Choice<'a> {
        Choice {
            rank: rank,
            subrank: subrank,
            string: SortableStr(string)
        }
    }

    pub fn from(string: &'a str) -> Choice<'a> {
        Choice { string: SortableStr(string), ..Default::default() }
    }

    pub fn start(&self) -> usize {
        self.subrank
    }

    pub fn end(&self) -> usize {
        self.rank + self.subrank
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
    let last_index = query.len() - 1;
    let partial: String = query[0..last_index].iter()
        .map(|ch| regex::escape(&ch.to_string()))
        .map(|esc| format!("{}.*?", esc)) // (?i) for case insensitive
        .collect();

    format!("(?i){}{}", partial, regex::escape(&query[last_index].to_string()))
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
            Choice::new(4, 11, "/some/path/user_group.rs"),
            Choice::new(4, 15, "/some/path/api_user.rs"),
            Choice::new(4, 18, "/some/deeper/path/users.rs"),
            Choice::new(5, 11, "/some/path/use_remote.rs"),
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_is_case_insensitive() {
        let query = ['U', 's', 'R'];
        let expected = vec![
            Choice::new(4, 11, "/some/path/user_group.rs"),
            Choice::new(4, 15, "/some/path/api_user.rs"),
            Choice::new(4, 18, "/some/deeper/path/users.rs"),
            Choice::new(5, 11, "/some/path/use_remote.rs"),
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_reserved_chars() {
        let query = ['?', '*', '.'];
        let expected = vec![
            Choice::new(3, 8, "reserved?*.rs")
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_special_chars() {
        let query = ['ß', '💣'];
        let expected = vec![
            Choice::new(9, 0, "ßℝ💣")
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
