use regex;

// Idea taken from:
//   http://blog.amjith.com/fuzzyfinder-in-10-lines-of-python
pub fn finder<'a>(list: &'a [&'a str], query: &'a [char]) -> Vec<&'a str> {
    let pattern: String = query.iter()
                        .map(|ch| format!("(?i){}.*?", ch)) // (?i) for case insensitive
                        .collect();
    let re = regex::Regex::new(&pattern).unwrap();

    let mut suggestions: Vec<(usize, usize, &str)> = list.iter()
        .map(|item| (re.find(item), *item))
        .filter(|tup| tup.0.is_some())
        .map(|(m, item)| (m.unwrap(), item))
        .map(|(m, item)| (m.as_str().len(), m.start(), item))
        .collect();

    suggestions.sort();
    suggestions.iter().map(|&(_, _, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex;

    #[test]
    fn it_gets_best_matches() {
        let list = vec![
            "/some/path/api_user.rs",
            "/some/path/user_group.rs",
            "foobar.rs",
        ];
        let query = vec!['u', 's', 'e', 'r'];
        let expected = vec![
            "/some/path/user_group.rs",
            "/some/path/api_user.rs",
        ];

        assert_eq!(expected, finder(&list, &query));
    }

    #[test]
    fn it_returns_the_same_on_empty_query() {
        let list = vec![
            "/some/path/api_user.rs",
            "/some/path/user_group.rs",
            "foobar.rs",
        ];
        let query = vec![];
        let expected = vec![
            "/some/path/api_user.rs",
            "/some/path/user_group.rs",
            "foobar.rs",
        ];

        assert_eq!(expected, finder(&list, &query));
    }
}
