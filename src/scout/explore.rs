use regex::Regex;
use super::choice::Choice;
use super::pattern::Pattern;

pub fn explore<'a, 'b>(list: &'a [&'a str], query: &'b [char]) -> Vec<Choice> {
    if query.is_empty() {
        return list.iter().map(|&text| text.to_string().into()).collect::<Vec<Choice>>()
    }

    let pattern = Pattern::build(query);
    let re = Regex::new(&pattern.to_string()).unwrap();

    let mut choices: Vec<Choice> = list.iter()
        .map(|text| filter(&re, text.to_string()))
        .filter(|choice| choice.is_some())
        .map(|choice| choice.unwrap())
        .collect();

    choices.sort();

    choices
}

fn filter<'b>(re: &'b Regex, text: String) -> Option<Choice> {
    let text = &text;
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
                        let choice = (text.to_string(), matching.start() + index, matching.end() + index).into();
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
        let text = "axby".to_string();

        assert_eq!(None, filter(&re, text));
    }

    #[test]
    fn filter_returns_the_only_choice_possible() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyc";
        let expected = Some(Choice::new(text.to_string(), 0, 5));

        assert_eq!(expected, filter(&re, text.to_string()));
    }

    #[test]
    fn filter_returns_the_best_choice_possible_on_repeating_matches() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        // the second match, after the "/",
        // scores better because it's shorter
        let text = "axbyc/abyc"; 
        let expected = Some(Choice::new(text.to_string(), 6, 10));

        assert_eq!(expected, filter(&re, text.to_string()));
    }

    #[test]
    fn filter_returns_the_best_choice_possible_on_overlapping_matches() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyabzcc";
        let expected = Some(Choice::new(text.to_string(), 4, 8));

        assert_eq!(expected, filter(&re, text.to_string()));
    }

    #[test]
    fn it_gets_best_matches() {
        let query = ['u', 's', 'r'];
        let expected = vec![
            Choice::new("/some/path/user_group.rs".to_string(), 11, 15),
            Choice::new("/some/path/api_user.rs".to_string(), 15, 19),
            Choice::new("/some/deeper/path/users.rs".to_string(), 18, 22),
            Choice::new("/some/path/use_remote.rs".to_string(), 11, 16),
        ];

        assert_eq!(expected, explore(&LIST, &query));
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

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_reserved_chars() {
        let query = ['?', '*', '.'];
        let expected = vec![
            Choice::new("reserved?*.rs".to_string(), 8, 11)
        ];

        assert_eq!(expected, explore(&LIST, &query));
    }

    #[test]
    fn it_takes_special_chars() {
        let query = ['ß', '💣'];
        let expected = vec![
            Choice::new("ßℝ💣".to_string(), 0, 9)
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
