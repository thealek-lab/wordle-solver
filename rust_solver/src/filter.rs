pub const UNKNOWN_CHAR: char = '_';
pub const FILTER_LENGTH: usize = 5;
pub const EMPTY_FILTER: &str = "_____";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub guess_str: String,
    pub feedback_str: String,

    pub include: Vec<char>,
    pub exclude: Vec<char>,
    pub is_empty: bool,
}

impl Filter {
    pub fn new(guess_str: &str, feedback_str: &str) -> Result<Self, String> {
        assert!(
            guess_str.len() == FILTER_LENGTH,
            "Filter guess string '{guess_str}' is not 5 characters!"
        );

        assert!(
            guess_str.chars().all(char::is_alphabetic),
            "Filter guess string '{guess_str}' is not alphabetic"
        );
        let guess = guess_str.as_bytes();

        let mut feedback = feedback_str.chars().collect::<Vec<_>>();
        feedback.resize(FILTER_LENGTH, UNKNOWN_CHAR);
        assert!(
            feedback.len() == FILTER_LENGTH,
            "Filter string '{feedback_str}' is not 5 characters long"
        );

        let mut include = vec![];
        let mut exclude: Vec<char> = vec![];
        for (i, feedback_char) in feedback.iter().enumerate() {
            let guess_char = guess[i] as char;
            if guess_char.is_ascii_uppercase() {
                include.push(guess_char);
                exclude.push(UNKNOWN_CHAR)
            } else if *feedback_char == UNKNOWN_CHAR {
                include.push(UNKNOWN_CHAR);
                exclude.push(guess_char.to_ascii_lowercase())
            } else {
                include.push(guess_char);
                exclude.push(UNKNOWN_CHAR)
            }
        }

        let include_str: String = include.iter().collect();

        Ok(Self {
            guess_str: guess_str.to_string(),
            feedback_str: feedback_str.to_string(),
            include,
            exclude,
            is_empty: include_str == EMPTY_FILTER,
        })
    }

    pub fn make_filter(filter_str: &str) -> Result<Self, String> {
        assert!(
            filter_str.len() >= FILTER_LENGTH,
            "Filter string '{filter_str}' is less than 5 characters!"
        );

        let guess_str = &filter_str[..FILTER_LENGTH];
        let feedback = &filter_str[FILTER_LENGTH..];

        Self::new(guess_str, feedback)
    }

    pub fn make_filter_func(&self) -> impl Fn(&str) -> bool + '_ {
        move |maybe_sol| {
            assert!(
                maybe_sol.chars().all(char::is_alphabetic),
                "Possible solution {maybe_sol} is not alphabetic"
            );
            assert!(
                maybe_sol.chars().all(char::is_uppercase),
                "Possible solution {maybe_sol} is not uppercase"
            );
            assert_eq!(
                maybe_sol.chars().count(),
                FILTER_LENGTH,
                "Possible solution {maybe_sol} is not 5 characters long"
            );

            let mut solution = maybe_sol.chars().collect::<Vec<_>>();

            for (index, &include_char) in self.include.iter().enumerate() {
                if include_char != UNKNOWN_CHAR && include_char.is_uppercase() {
                    if solution[index] != include_char {
                        return false;
                    }
                    solution[index] = UNKNOWN_CHAR;
                }
            }

            for (index, &include_char) in self.include.iter().enumerate() {
                if include_char != UNKNOWN_CHAR && include_char.is_lowercase() {
                    let upper = include_char.to_uppercase().next().unwrap();
                    if solution[index] == upper {
                        return false;
                    }

                    let Some(position) = solution.iter().position(|&character| character == upper)
                    else {
                        return false;
                    };
                    solution[position] = UNKNOWN_CHAR;
                }
            }

            for &exclude_char in &self.exclude {
                if exclude_char != UNKNOWN_CHAR {
                    let upper = exclude_char.to_uppercase().next().unwrap();
                    if solution.contains(&upper) {
                        return false;
                    }
                }
            }

            true
        }
    }

    pub fn make_hint(guess_str: &str, solution_str: &str) -> Result<Self, String> {
        let guess = guess_str.to_uppercase();
        assert!(
            guess.chars().all(char::is_alphabetic),
            "Guess string '{guess}' is not alphabetic!"
        );
        assert!(
            guess.chars().count() == FILTER_LENGTH,
            "Guess string '{guess}' is not 5 characters long!"
        );

        let solution_str = solution_str.to_uppercase();
        assert!(
            solution_str.chars().all(char::is_alphabetic),
            "Solution string '{solution_str}' is not alphabetic!"
        );
        assert!(
            solution_str.chars().count() == FILTER_LENGTH,
            "Solution string '{solution_str}' is not 5 characters long!"
        );

        let guess_chars: Vec<char> = guess.chars().collect();
        let solution: Vec<char> = solution_str.chars().collect();
        let mut remaining = solution.clone();
        let mut guess_out = vec![];
        let mut feedback = vec![];

        for (index, guess_char) in guess_chars.iter().enumerate() {
            if *guess_char == solution[index] {
                remaining[index] = UNKNOWN_CHAR;
                guess_out.push(*guess_char);
                feedback.push(*guess_char);
            } else {
                guess_out.push(UNKNOWN_CHAR);
                feedback.push(UNKNOWN_CHAR);
            }
        }

        for (index, guess_char) in guess_chars.iter().enumerate() {
            if guess_out[index] == UNKNOWN_CHAR {
                if let Some(pos) = remaining.iter().position(|&b| b == *guess_char) {
                    remaining[pos] = UNKNOWN_CHAR;
                    feedback[index] = guess_char.to_ascii_lowercase();
                }
                guess_out[index] = guess_char.to_ascii_lowercase();
            }
        }

        let guess_out: String = guess_out.into_iter().collect();
        let feedback: String = feedback.into_iter().collect();

        Self::new(&guess_out, &feedback)
    }
}

impl std::fmt::Display for Filter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.feedback_str != EMPTY_FILTER {
            write!(formatter, "{}{}", self.guess_str, self.feedback_str)
        } else {
            write!(formatter, "{}", self.guess_str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_matches_python_cases() {
        let test_cases = [
            ("ABACK", "BREAD", "abackab___"),
            ("ABACK", "BREAK", "abacKab__K"),
            ("ABACK", "BREAM", "abackab___"),
            ("ABAMK", "BREAD", "abamkab___"),
            ("ABAMK", "BREAK", "abamKab__K"),
            ("ABAMK", "BREAM", "abamkab_m_"),
            ("WHOSE", "WORSE", "WhoSEW_oSE"),
            ("WORSE", "WHOSE", "WorSEWo_SE"),
        ];

        for (guess, solution, expected) in test_cases {
            assert_eq!(
                Filter::make_hint(guess, solution).unwrap().to_string(),
                expected
            );
        }
    }

    #[test]
    fn filter_matches_python_cases() {
        let test_cases = [
            ("BACxx", "BACON", true),
            ("xALEx", "VALET", true),
            ("xALEx", "PALER", true),
            ("xALEx", "BALER", true),
            ("xALEx", "PALED", true),
            ("xALEb____b", "BALER", true),
            ("xALEb____b", "BALED", true),
            ("xalek_alek", "SALEK", false),
            ("xalek_alek", "ALIKE", true),
            ("xalek_alek", "ANKLE", true),
            ("xalek_alek", "FLAKE", true),
            ("xalek_alek", "LEAKY", true),
            ("xalek_alek", "SLAKE", true),
            ("xkela_kela", "SALEK", true),
            ("xkela_kela", "ALIKE", true),
            ("xkela_kela", "FLAKE", true),
            ("xkela_kela", "LEAKY", true),
            ("xkela_kela", "SLAKE", true),
            ("xkela_kela", "LATKE", true),
            ("AlekxAlek", "ANKLE", true),
            ("abackab", "BREAD", true),
            ("abacKab", "BREAK", true),
            ("abackab", "BREAM", true),
            ("abacKab", "ABACK", false),
            ("abacK__ac_", "ABACK", false),
            ("abamkab", "BREAD", true),
            ("abamKab", "BREAK", true),
            ("abamkab_m", "BREAM", true),
            ("BREAd____d", "BREAD", false),
            ("BREAd", "BREAK", true),
            ("BREAd", "BREAM", true),
            ("WhoSE", "WORSE", false),
            ("WhoSE__o", "WORSE", true),
            ("WorSE", "WHOSE", false),
            ("WorSE_o", "WHOSE", true),
        ];

        for (filter_str, candidate, expected) in test_cases {
            let filter = Filter::make_filter(filter_str).unwrap();
            eprintln!("Filter: {filter_str}, Candidate: {candidate}, Expected: {expected}");
            assert_eq!(
                filter.make_filter_func()(candidate),
                expected,
                "Filter {filter_str}"
            );
        }
    }
}
