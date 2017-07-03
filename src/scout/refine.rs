use regex::Regex;
use choice::Choice;

/// Given a `Regex` and a `&str`, determine if is a valid `Choice`.
///
/// If `None` is returned it means that the text doesn't have match with the `Regex`, so it
/// should be descarted.
///
/// This is the main algorithm to detect if a text matches against a set of chars, so lets explain
/// it a little bit:
///
/// Imagine that you have the Regex based on the pattern:
///
/// ```rust,ignore
/// extern crate regex;
/// use regex::Regex;
///
/// let re = Regex::new("(?i)a.*?b.*?c").unwrap();
/// ```
///
/// Which means: check if a string has an `a`, then anything, then a `b`, then anything and then
/// `c` and be case insensitive, please.
///
/// Now imagine that you have the text:
///
/// ```rust,ignore
/// let text = "a/a/b/c.rs"
/// ```
///
/// We need to note that when doing a fuzzy search, we want the shortest match possible so we can
/// narrow to the best possible match.
///
/// Now, returning to the example, if we run the regex agains the string, it will match, but not
/// with the shortest match. It will match this section:
///
/// ```rust,ignore
/// let matching = "(a/a/b/c).rs"
/// ```
///
/// But the best possible match would be:
///
/// ```rust,ignore
/// let best_matching = "a/(a/b/c).rs"
/// ```
///
/// How can we get the best match? By getting all the possible matches in a string.
///
/// The idea is that once we get a match, we cut the string where the match starts and then we try
/// this new string against the `Regex` again. If it matches, then we have a shorter match. If not,
/// then we can't find any other match.
///
/// In the example, we would do the following:
///
/// * We run `"a/a/b/c.rs"` against `(?i)a.*?b.*?c` and we get the match `(a/a/b/c).rs`.
/// * We remove the starting char of the match, in this case the first `a`, index `0`. This gives
///   us the string `"/a/b/c.rs"`.
/// * We run `"/a/b/c.rs"` against `(?i)a.*?b.*?c` and we get the match `/(a/b/c).rs`.
/// * We remove the starting char of the match, in this case the second `a`, index `2`. This gives
///   us the string "`/b/c.rs`".
/// * We run `"/b/c.rs"` against `(?i)a.*?b.*?c` and we don't get any match. This is the end of the
/// checks.
/// * Now we have two matches, two possible choices. We select the shortest which is the one
///   matching `"a/(a/b/c).rs"`.
///
/// The `refine` function does all of this for us:
///
/// ```rust,ignore
/// extern crate regex;
/// extern crate scout;
///
/// use regex::Regex;
/// use scout;
///
/// let re = Regex::new("(?i)a.*?b.*?c").unwrap();
/// let text = "a/a/b/c.rs";
///
/// match scout::refine(&re, text) {
///   Some(choice) => {
///     let debug = format!("{} ({}, {})", choice.to_string(), choice.start(), choice.end());
///
///     assert_eq!("a/a/b/c.rs (2, 7)", debug);
///   }
///   None => panic!("It should match")
/// };
/// ```
pub fn refine(re: &Regex, text: &str) -> Option<Choice> {
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

    // A Choice compares with others using its Score, so we know
    // we are getting the shortest match when doing the `min()` call.
    matches.into_iter().min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pattern::Pattern;

    #[test]
    fn returns_none_when_there_is_no_match() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::new(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axby";

        assert_eq!(None, refine(&re, text));
    }

    #[test]
    fn returns_the_only_choice_possible() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::new(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyc";
        let expected = Some(Choice::new(text.to_string(), 0, 5));

        assert_eq!(expected, refine(&re, text));
    }

    #[test]
    fn returns_the_best_choice_possible_on_repeating_matches() {
        let query = vec!['a', 'b', 'c'];
        let pattern = Pattern::new(&query);
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
        let pattern = Pattern::new(&query);
        let re = Regex::new(&pattern.to_string()).unwrap();
        let text = "axbyabzcc";
        let expected = Some(Choice::new(text.to_string(), 4, 8));

        assert_eq!(expected, refine(&re, text));
    }
}
