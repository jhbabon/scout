extern crate regex;

use regex::Regex;

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
pub fn explore<'a>(list: &'a [&'a str], query: &'a [char]) -> Vec<&'a str> {
    let pattern = build_pattern(query);
    let re = Regex::new(&pattern).unwrap();

    let mut suggestions: Vec<(usize, usize, &str)> = list.iter()
        .map(|item| (re.find(item), *item))
        .filter(|tup| tup.0.is_some())
        .map(|(m, item)| (m.unwrap(), item))
        .map(|(m, item)| (m.as_str().len(), m.start(), item))
        .collect();

    suggestions.sort();
    suggestions.iter().map(|&(_, _, item)| item).collect()
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

    const LIST: [&'static str; 6] = [
        "/some/deeper/path/users.rs",
        "/some/path/api_user.rs",
        "/some/path/user_group.rs",
        "foobar.rs",
        "reserved?*.rs",
        "ßℝ💣",
    ];

    #[test]
    fn it_gets_best_matches() {
        let query = ['u', 's', 'r'];
        let expected = vec![
            "/some/path/user_group.rs",
            "/some/path/api_user.rs",
            "/some/deeper/path/users.rs",
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_is_case_insensitive() {
        let query = ['U', 's', 'R'];
        let expected = vec![
            "/some/path/user_group.rs",
            "/some/path/api_user.rs",
            "/some/deeper/path/users.rs",
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_reserved_chars() {
        let query = ['?', '*', '.'];
        let expected = vec![
            "reserved?*.rs"
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_special_chars() {
        let query = ['ß', '💣'];
        let expected = vec![
            "ßℝ💣"
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_returns_the_same_on_empty_query() {
        let query = [];
        let expected: Vec<&str> = LIST.iter().map(|&s| s).collect();

        assert_eq!(expected, explore(&LIST, &query));
    }
}
