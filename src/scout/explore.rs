extern crate regex;

use self::regex::Regex;
use super::choice::Choice;
use super::pattern::Pattern;

// Idea taken from:
//   http://blog.amjith.com/fuzzyfinder-in-10-lines-of-python
//
// TODO: Return Result to handle errors
pub fn explore<'a, 'b>(list: &'a [&'a str], query: &'b [char]) -> Vec<Choice<'a>> {
    if query.is_empty() {
        return list.iter().map(|&text| text.into()).collect::<Vec<Choice>>()
    }

    let pattern = Pattern::build(query);
    let re = Regex::new(&pattern.to_string()).unwrap();

    let mut choices: Vec<Choice> = list.iter()
        .map(|text| filter(&re, text))
        .filter(|choice| choice.is_some())
        .map(|choice| choice.unwrap())
        .collect();

    choices.sort();

    choices
}

fn filter<'a, 'b>(re: &'b Regex, text: &'a str) -> Option<Choice<'a>> {
    let mut indexes = text.char_indices().map(|(index, _)| index);
    let mut matches: Vec<Choice> = vec![];
    let mut last_match = 0;

    loop {
        let last = last_match;
        // We don't need to iterate over each index, just the ones
        // after the last match from the regex
        let mut iter = indexes.by_ref().skip_while(|&index| last > index);

        match iter.next() {
            Some(index) => {
                let ma = re.find(&text[index..]);
                match ma {
                    Some(matching) => {
                        last_match = matching.start();
                        let choice = (text, matching.start() + index, matching.end() + index).into();
                        matches.push(choice)
                    },
                    None => break
                }
            }
            None => break
        }
    }

    matches.into_iter().min()
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
    fn filter_returns_none_when_there_is_no_match() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axby";

        assert_eq!(None, filter(&re, text));
    }

    #[test]
    fn filter_returns_the_only_choice_possible() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyc";
        let expected = Some(Choice::new(text, 0, 5));

        assert_eq!(expected, filter(&re, text));
    }

    #[test]
    fn filter_returns_the_best_choice_possible_on_repeating_matches() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        // the second match, after the "/",
        // scores better because it's shorter
        let text = "axbyc/abyc"; 
        let expected = Some(Choice::new(text, 6, 10));

        assert_eq!(expected, filter(&re, text));
    }

    #[test]
    fn filter_returns_the_best_choice_possible_on_overlapping_matches() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyabzcc";
        let expected = Some(Choice::new(text, 4, 8));

        assert_eq!(expected, filter(&re, text));
    }

    #[test]
    fn it_gets_best_matches() {
        let query = ['u', 's', 'r'];
        let expected = vec![
            Choice::new("/some/path/user_group.rs", 11, 15),
            Choice::new("/some/path/api_user.rs", 15, 19),
            Choice::new("/some/deeper/path/users.rs", 18, 22),
            Choice::new("/some/path/use_remote.rs", 11, 16),
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_is_case_insensitive() {
        let query = ['U', 's', 'R'];
        let expected = vec![
            Choice::new("/some/path/user_group.rs", 11, 15),
            Choice::new("/some/path/api_user.rs", 15, 19),
            Choice::new("/some/deeper/path/users.rs", 18, 22),
            Choice::new("/some/path/use_remote.rs", 11, 16),
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_reserved_chars() {
        let query = ['?', '*', '.'];
        let expected = vec![
            Choice::new("reserved?*.rs", 8, 11)
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_special_chars() {
        let query = ['ß', '💣'];
        let expected = vec![
            Choice::new("ßℝ💣", 0, 9)
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
