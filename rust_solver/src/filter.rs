pub const UNKNOWN_CHAR: char = '_';
pub const FILTER_LENGTH: usize = 5;
pub const EMPTY_FILTER: &str = "_____";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
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

        let include = &filter_str[..FILTER_LENGTH];
        let exclude = &filter_str[FILTER_LENGTH..];

        Self::new(include, exclude)
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

    pub fn make_hint(guess: &str, solution: &str) -> Result<Self, String> {
        let guess = guess.to_uppercase();
        assert!(
            guess.chars().all(char::is_alphabetic),
            "Guess string '{guess}' is not alphabetic!"
        );
        assert_eq!(
            guess.chars().count(),
            FILTER_LENGTH,
            "Guess string '{guess}' is not 5 characters long!"
        );

        let solution = solution.to_uppercase();
        assert!(
            solution.chars().all(char::is_alphabetic),
            "Solution string '{solution}' is not alphabetic!"
        );
        assert_eq!(
            solution.chars().count(),
            FILTER_LENGTH,
            "Solution string '{solution}' is not 5 characters long!"
        );

        let guess_chars = guess.chars().collect::<Vec<_>>();
        let mut remaining = solution.chars().collect::<Vec<_>>();
        let mut include = [UNKNOWN_CHAR; FILTER_LENGTH];
        let mut exclude = [UNKNOWN_CHAR; FILTER_LENGTH];

        for index in 0..FILTER_LENGTH {
            if guess_chars[index] == remaining[index] {
                include[index] = guess_chars[index].to_uppercase().next().unwrap();
                remaining[index] = UNKNOWN_CHAR;
            }
        }

        for index in 0..FILTER_LENGTH {
            let character = guess_chars[index];
            let position = remaining
                .iter()
                .position(|&candidate| candidate == character);
            match position {
                Some(position) => {
                    if include[index] != character {
                        include[index] = character.to_lowercase().next().unwrap();
                        remaining[position] = UNKNOWN_CHAR;
                    }
                }
                None => {
                    if include[index] != character {
                        exclude[index] = character.to_lowercase().next().unwrap();
                    }
                }
            }
        }

        Self::new(
            &include.iter().collect::<String>(),
            &exclude.iter().collect::<String>(),
        )
    }
}

impl std::fmt::Display for Filter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let include = self.include.iter().collect::<String>();
        let exclude = self.exclude.iter().collect::<String>();
        if exclude != EMPTY_FILTER {
            write!(formatter, "{include}^{exclude}")
        } else {
            write!(formatter, "{include}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_matches_python_cases() {
        let test_cases = [
            ("ABACK", "BREAD", "ab___^__ack"),
            ("ABACK", "BREAK", "ab__K^__ac_"),
            ("ABACK", "BREAM", "ab___^__ack"),
            ("ABAMK", "BREAD", "ab___^__amk"),
            ("ABAMK", "BREAK", "ab__K^__am_"),
            ("ABAMK", "BREAM", "ab_m_^__a_k"),
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
            ("BAC__", "BACON", true),
            ("_ALE", "VALET", true),
            ("_ALE", "PALER", true),
            ("_ALE", "BALER", true),
            ("_ALE", "PALED", true),
            ("_ALEb", "BALER", true),
            ("_ALEb", "BALED", true),
            ("_alek", "SALEK", false),
            ("_alek", "ALIKE", true),
            ("_alek", "ANKLE", true),
            ("_alek", "FLAKE", true),
            ("_alek", "LEAKY", true),
            ("_alek", "SLAKE", true),
            ("_kela", "SALEK", true),
            ("_kela", "ALIKE", true),
            ("_kela", "FLAKE", true),
            ("_kela", "LEAKY", true),
            ("_kela", "SLAKE", true),
            ("_kela", "LATKE", true),
            ("Alek", "ANKLE", true),
            ("ab___^__ack", "BREAD", true),
            ("ab__K^__ac_", "BREAK", true),
            ("ab___^__ack", "BREAM", true),
            ("ab___^__ack", "ABACK", false),
            ("ab__K^__ac_", "ABACK", false),
            ("ab___^__amk", "BREAD", true),
            ("ab__K^__am_", "BREAK", true),
            ("ab_m_^__a_k", "BREAM", true),
            ("BREA_^____d", "BREAD", false),
            ("BREA_^____d", "BREAK", true),
            ("BREA_^____d", "BREAM", true),
        ];

        for (filter_str, candidate, expected) in test_cases {
            let filter = Filter::make_filter(filter_str).unwrap();
            assert_eq!(
                filter.make_filter_func()(candidate),
                expected,
                "Filter {filter_str}"
            );
        }
    }
}
