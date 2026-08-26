use crate::filter::Filter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guess {
    pub maybe_sol: String,
    pub is_in_remaining_sol_set: bool,
    pub hint: Filter,
    pub remaining_sol_set: Vec<String>,
    pub remaining_cnt: usize,
}

impl Guess {
    pub fn new(maybe_sol: String, hint: Filter, remaining_sol_set: Vec<String>) -> Self {
        let is_in_remaining_sol_set = remaining_sol_set.contains(&maybe_sol);
        let remaining_cnt = remaining_sol_set.len();

        Self {
            maybe_sol,
            is_in_remaining_sol_set,
            hint,
            remaining_sol_set,
            remaining_cnt,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuessSet {
    pub guess_str: String,
    pub is_in_tot_sol_set: bool,
    pub is_in_remaining_sol_set: bool,
    pub guesses: Vec<Guess>,
    pub num_sols: usize,
    pub remaining_cnt: f64,
    pub remaining_avg: f64,
}

impl GuessSet {
    pub fn new(
        guess_str: String,
        num_sols: usize,
        is_in_tot_sol_set: bool,
        is_in_remaining_sol_set: bool,
    ) -> Self {
        Self {
            guess_str,
            is_in_tot_sol_set,
            is_in_remaining_sol_set,
            guesses: Vec::new(),
            num_sols,
            remaining_cnt: f64::MAX,
            remaining_avg: f64::INFINITY,
        }
    }

    pub fn add_guess(&mut self, guess: Guess) {
        self.guesses.push(guess);
        if self.remaining_cnt == f64::MAX {
            self.remaining_cnt = 0.0;
        }

        let new_sols = self.guesses.last().unwrap().remaining_sol_set.len() as f64;
        if new_sols > 0.0 {
            self.remaining_cnt += 0.5 + new_sols / 2.0;
        }
        self.remaining_avg = self.remaining_cnt / self.num_sols as f64;
    }

    pub fn is_better_than(&self, other: &Self, verbosity: u32) -> bool {
        if self.remaining_cnt < other.remaining_cnt {
            return true;
        }

        if self.guess_str != other.guess_str && self.remaining_cnt == other.remaining_cnt {
            if self.is_in_remaining_sol_set && !other.is_in_remaining_sol_set {
                if verbosity >= 1 {
                    println!(
                        "   Guess {} TIED with {}: preferred because one of remaining solutions",
                        self, other
                    );
                }
                return true;
            }

            if self.is_in_tot_sol_set && !other.is_in_tot_sol_set {
                if verbosity >= 1 {
                    println!(
                        "   Guess {} TIED with {}: preferred because one of official solutions",
                        self, other
                    );
                }
                return true;
            }
        }

        false
    }
}

impl std::fmt::Display for GuessSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guess_type = if self.is_in_remaining_sol_set {
            "+"
        } else if self.is_in_tot_sol_set {
            ""
        } else {
            "*"
        };

        write!(formatter, "{}{}", self.guess_str, guess_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guess(maybe_sol: &str, remaining_sol_set: &[&str]) -> Guess {
        Guess::new(
            maybe_sol.to_owned(),
            Filter::make_filter("_____").unwrap(),
            remaining_sol_set
                .iter()
                .map(|solution| (*solution).to_owned())
                .collect(),
        )
    }

    #[test]
    fn scores_and_prefers_remaining_solution_on_ties() {
        let mut remaining_guess = GuessSet::new("raise".to_owned(), 2, true, true);
        remaining_guess.add_guess(guess("raise", &["crane", "slate"]));

        let mut other_guess = GuessSet::new("audio".to_owned(), 2, false, false);
        other_guess.add_guess(guess("audio", &["crane", "slate"]));

        assert_eq!(remaining_guess.remaining_cnt, 1.5);
        assert_eq!(remaining_guess.remaining_avg, 0.75);
        assert_eq!(remaining_guess.to_string(), "raise+");
        assert!(remaining_guess.is_better_than(&other_guess, 0));
        assert_eq!(other_guess.to_string(), "audio*");
    }
}
