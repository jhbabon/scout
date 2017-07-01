use regex::Regex;
use num_cpus;
use futures::future::{Future, join_all};
use futures_cpupool::CpuPool;

use super::choice::Choice;
use super::pattern::Pattern;
use super::refine;

pub struct Scout {
    list: Vec<String>,
    chunks: Vec<Vec<String>>,
    pool: CpuPool,
}

impl Scout {
    pub fn new<'a>(list: Vec<&'a str>) -> Self {
        let size = num_cpus::get();
        let chunk_size = if list.len() < size {
            list.len()
        } else {
            list.len() / size
        };

        let list: Vec<String> = list.iter().map(|t| String::from(*t)).collect();

        let chunks = list.chunks(chunk_size)
            .map(|section| Vec::from(section))
            .collect();

        let pool = CpuPool::new(size);

        Self { list, chunks, pool }
    }

    pub fn explore<'b>(&self, query: &'b [char]) -> Vec<Choice> {
        if query.is_empty() {
            return self.list
                .iter()
                .cloned()
                .map(|text| text.into())
                .collect::<Vec<Choice>>();
        }

        let re = self.regex(query);

        let futures = self.chunks
            .iter()
            .cloned()
            .map(|lines| {
                let reg = re.clone();

                self.pool.spawn_fn(move || {
                    let choices: Vec<Option<Choice>> =
                        lines.into_iter().map(|line| refine(&reg, line)).collect();
                    let result: Result<Vec<Option<Choice>>, ()> = Ok(choices);
                    result
                })
            })
            .collect::<Vec<_>>();

        let mut choices: Vec<Choice> = join_all(futures)
            .map(|values| {
                values
                    .iter()
                    .cloned()
                    .flat_map(|choices| choices)
                    .filter(|choice| choice.is_some())
                    .map(|choice| choice.unwrap())
                    .collect::<Vec<Choice>>()
            })
            .wait()
            .unwrap();

        choices.sort();

        choices
    }

    fn regex<'b>(&self, query: &'b [char]) -> Regex {
        let pattern = Pattern::build(query);

        Regex::new(&pattern.to_string()).unwrap()
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
