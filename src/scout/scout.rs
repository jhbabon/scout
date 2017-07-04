use regex::Regex;
use num_cpus;
use futures::future::{Future, join_all};
use futures_cpupool::CpuPool;

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
pub struct Scout {
    list: Vec<String>,
    chunks: Vec<Vec<String>>,
    pool: CpuPool,
}

impl Scout {
    /// Create a new Scout instance with a list of strings
    pub fn new(list: Vec<&str>) -> Self {
        let size = num_cpus::get();
        let chunk_size = if list.len() < size {
            list.len()
        } else {
            list.len() / size
        };

        let list: Vec<String> = list.into_iter().map(|t| String::from(t)).collect();

        let chunks = list.chunks(chunk_size).map(Vec::from).collect();

        let pool = CpuPool::new(size);

        Self { list, chunks, pool }
    }

    /// Search for the choices that match a query, sorted by best match first.
    ///
    /// If the query is empty, it returns all the choices with the original order of the items.
    pub fn explore<'a>(&self, query: &'a [char]) -> Vec<Choice> {
        if query.is_empty() {
            return self.list
                .iter()
                .cloned()
                .map(|text| text.into())
                .collect::<Vec<Choice>>();
        }

        let re = match self.regex(query) {
            Ok(r) => r,
            Err(e) => panic!("{:?}", e),
        };

        // Let's parallelize the search in different threads, one per chunk of lines.
        let futures = self.chunks
            .iter()
            .cloned()
            .map(|lines| {
                let reg = re.clone();

                self.pool.spawn_fn(move || {
                    let choices: Vec<Option<Choice>> =
                        lines.into_iter().map(|line| refine(&reg, &line)).collect();
                    let result: Result<Vec<Option<Choice>>, ()> = Ok(choices);

                    result
                })
            })
            .collect::<Vec<_>>();

        let waiting = join_all(futures)
            .map(|values| {
                values
                    .iter()
                    .cloned()
                    .flat_map(|choices| choices)
                    .filter_map(|choice| choice)
                    .collect::<Vec<Choice>>()
            })
            .wait();

        let mut choices: Vec<Choice> = match waiting {
            Ok(values) => values,
            Err(_) => vec![],
        };

        choices.sort();

        choices
    }

    /// Get a Regex from a list of chars.
    fn regex<'a>(&self, query: &'a [char]) -> Result<Regex, Error> {
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
