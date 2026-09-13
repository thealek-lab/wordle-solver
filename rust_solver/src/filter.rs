pub const WORD_LENGTH: usize = 5;

type WordChars = [char; WORD_LENGTH];

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
/// Represents a single clue for a letter in a guess, which can be green, yellow, or gray (underscore).
enum SingleClue {
    Green = 2,
    Yellow = 1,
    Gray = 0,
}

impl SingleClue {
    const UNKNOWN_CHAR: char = '_';
    const GREEN_CHAR: char = 'g';
    const YELLOW_CHAR: char = 'y';
}

impl std::fmt::Display for SingleClue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingleClue::Green => write!(formatter, "{}", Self::GREEN_CHAR),
            SingleClue::Yellow => write!(formatter, "{}", Self::YELLOW_CHAR),
            SingleClue::Gray => write!(formatter, "{}", Self::UNKNOWN_CHAR),
        }
    }
}
/// Represents a word clue value, which is a number between 0 and 242 (3^5 - 1) that encodes the possible clues for a guess.
type WordClueVal = usize;
type CharClues = [SingleClue; WORD_LENGTH];

#[derive(Debug)]
/// Represents a set of clues for a guess, which can be green, yellow, or gray (underscore).
pub struct WordClue {
    word_clue: CharClues,
    val: WordClueVal,
}

impl WordClue {
    /// Creates a new WordClue from a string of clues, which can be green, yellow, or gray (underscore).
    fn new_from_clue_str(clues_str: &str) -> Self {
        assert!(
            clues_str.len() == WORD_LENGTH,
            "Clues string '{clues_str}' is not 5 characters long!"
        );

        let char_clues = clues_str
            .to_lowercase()
            .chars()
            .map(|c| match c {
                SingleClue::GREEN_CHAR => SingleClue::Green,
                SingleClue::YELLOW_CHAR => SingleClue::Yellow,
                SingleClue::UNKNOWN_CHAR => SingleClue::Gray,
                _ => panic!("Invalid clue character '{c}'"),
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self::new_from_char_clues(char_clues)
    }

    fn new_from_char_clues(char_clues: CharClues) -> Self {
        Self {
            word_clue: char_clues,
            val: Self::calc_val(char_clues),
        }
    }

    fn calc_val(clues: CharClues) -> usize {
        let mut val = 0;
        for clue in clues {
            val = val * 3 + clue as usize;
        }

        val
    }

    pub fn make_word_chars(word: &str) -> WordChars {
        assert!(
            word.len() == WORD_LENGTH,
            "Guess string '{word}' is not 5 characters long!"
        );

        assert!(
            word.chars().all(char::is_alphabetic),
            "Guess string '{word}' is not alphabetic!"
        );

        word.to_uppercase()
            .chars()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    pub fn new_from_guess_n_solution(guess_chars: WordChars, solution: &str) -> Self {
        let solution_chars = Self::make_word_chars(solution);

        let mut char_clues = [SingleClue::Gray; WORD_LENGTH];
        let mut remaining = [true; WORD_LENGTH];

        for (index, &guess_char) in guess_chars.iter().enumerate() {
            if guess_char == solution_chars[index] {
                char_clues[index] = SingleClue::Green;
                remaining[index] = false;
            }
        }

        for index in 0..WORD_LENGTH {
            if let SingleClue::Gray = char_clues[index]
                && let Some(position) = (0..WORD_LENGTH).find(|&position| {
                    remaining[position] && solution_chars[position] == guess_chars[index]
                })
            {
                char_clues[index] = SingleClue::Yellow;
                remaining[position] = false;
            }
        }

        Self::new_from_char_clues(char_clues)
    }

    pub fn new_from_guess_n_clue_str(guess_and_clue_str: &str) -> (WordChars, Self) {
        let str_len = guess_and_clue_str.len();
        assert!(
            str_len >= WORD_LENGTH,
            "Guess+clue string '{guess_and_clue_str}' is less than 5 characters long!"
        );
        assert!(
            str_len <= 2 * WORD_LENGTH,
            "Guess+clue string '{guess_and_clue_str}' is more than 10 characters long!"
        );

        let mut guess_and_clue_str = guess_and_clue_str.to_string();
        if str_len < 2 * WORD_LENGTH {
            let unknown_char_as_str = SingleClue::UNKNOWN_CHAR.to_string();
            // Add a bunch of underscores to the end of the string to make it 10 characters long
            guess_and_clue_str.push_str(&unknown_char_as_str.repeat(2 * WORD_LENGTH - str_len));
        }

        let (guess_str, clue_str) = guess_and_clue_str.split_at(WORD_LENGTH);
        let guess_chars = Self::make_word_chars(guess_str);
        (guess_chars, Self::new_from_clue_str(clue_str))
    }

    pub fn calc_val_from_guess_n_solution(guess: WordChars, solution: &[char]) -> WordClueVal {
        let mut pattern = [SingleClue::Gray as usize; WORD_LENGTH];
        let mut remaining = [true; WORD_LENGTH];

        for index in 0..WORD_LENGTH {
            if guess[index] == solution[index] {
                pattern[index] = SingleClue::Green as usize;
                remaining[index] = false;
            }
        }

        for index in 0..WORD_LENGTH {
            if pattern[index] == SingleClue::Gray as usize
                && let Some(position) = (0..WORD_LENGTH)
                    .find(|&position| remaining[position] && solution[position] == guess[index])
            {
                pattern[index] = SingleClue::Yellow as usize;
                remaining[position] = false;
            }
        }

        pattern.iter().fold(0, |key, &value| key * 3 + value)
    }

    pub fn is_match(&self, guess_chars: WordChars, solution_str: &str) -> bool {
        let solution_chars = Self::make_word_chars(solution_str);

        Self::calc_val_from_guess_n_solution(guess_chars, &solution_chars) == self.val
    }
}

impl std::fmt::Display for WordClue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for clue in &self.word_clue {
            write!(formatter, "{clue}")?
        }

        Ok(())
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
                WordClue::new_from_guess_n_solution(guess, solution)
                    .word_clue
                    .iter()
                    .map(|clue| clue.to_string())
                    .collect::<String>(),
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
            ("pooli_oo", "POOLS", false),
            ("oxxoxo__o", "POOLS", true),
            ("oxoxxo_o", "POOLS", false),
            ("oxOxxo", "POOLS", true),
            ("oxxooo__oo", "POOLS", false),
            ("BREAd____d", "POOLS", false),
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
