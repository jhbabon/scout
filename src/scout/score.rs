#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Score {
    match_length: usize,
    index: usize,
}

impl Score {
    pub fn new<'a>(match_start: usize, match_end: usize) -> Self {
        Score {
            index: match_start,
            match_length: match_end - match_start,
        }
    }
}

impl From<(usize, usize)> for Score {
    fn from(tuple: (usize, usize)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_a_new_score() {
        let expected = Score {
            index: 1,
            match_length: 2,
        };

        assert_eq!(expected, Score::new(1, 3));
    }

    #[test]
    fn it_builds_a_new_score_from_a_tuple() {
        let expected = Score {
            index: 1,
            match_length: 2,
        };

        assert_eq!(expected, (1, 3).into());
    }

    #[test]
    fn sort_match_length_are_better_scores() {
        let smaller = Score::new(0, 2);
        let bigger = Score::new(0, 3);

        assert!(smaller < bigger);
    }

    #[test]
    fn smaller_indexes_are_better_scores() {
        let smaller = Score::new(0, 2);
        let bigger = Score::new(1, 3);

        assert!(smaller < bigger);
    }

    #[test]
    fn match_length_scores_higher_than_index() {
        let smaller = Score::new(1, 3);
        let bigger = Score::new(0, 3);

        assert!(smaller < bigger);
    }
}
