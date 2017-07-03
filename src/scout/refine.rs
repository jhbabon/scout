use regex::Regex;
use choice::Choice;

pub fn refine(re: &Regex, text: &str) -> Option<Choice> {
    // let text = &text;
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
                        let choice = (
                            text.to_string(),
                            matching.start() + index,
                            matching.end() + index,
                        ).into();
                        matches.push(choice)
                    }
                    None => break,
                }
            }
            None => break,
        }
    }

    matches.into_iter().min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pattern::Pattern;

    #[test]
    fn returns_none_when_there_is_no_match() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axby";

        assert_eq!(None, refine(&re, text));
    }

    #[test]
    fn returns_the_only_choice_possible() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyc";
        let expected = Some(Choice::new(text.to_string(), 0, 5));

        assert_eq!(expected, refine(&re, text));
    }

    #[test]
    fn returns_the_best_choice_possible_on_repeating_matches() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        // the second match, after the "/",
        // scores better because it's shorter
        let text = "axbyc/abyc";
        let expected = Some(Choice::new(text.to_string(), 6, 10));

        assert_eq!(expected, refine(&re, text));
    }

    #[test]
    fn returns_the_best_choice_possible_on_overlapping_matches() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::build(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyabzcc";
        let expected = Some(Choice::new(text.to_string(), 4, 8));

        assert_eq!(expected, refine(&re, text));
    }
}
