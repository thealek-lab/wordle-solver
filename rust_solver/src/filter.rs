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
pub type WordClueVal = usize;
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
            ("ABACK", "BREAD", "yy___"),
            ("ABACK", "BREAK", "yy__g"),
            ("ABACK", "BREAM", "yy___"),
            ("ABAMK", "BREAD", "yy___"),
            ("ABAMK", "BREAK", "yy__g"),
            ("ABAMK", "BREAM", "yy_y_"),
            ("WHOSE", "WORSE", "g_ygg"),
            ("WORSE", "WHOSE", "gy_gg"),
        ];

        for (guess_str, solution, expected) in test_cases {
            let guess_chars = WordClue::make_word_chars(guess_str);
            assert_eq!(
                WordClue::new_from_guess_n_solution(guess_chars, solution)
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
            ("BACxxggg", "BACON", true),
            ("xALEx_ggg", "VALET", true),
            ("xALEx_ggg", "PALER", true),
            ("xALEx_ggg_", "BALER", true),
            ("xALEx_ggg", "PALED", true),
            ("xALEb_gggy", "BALER", true),
            ("xALEb_gggy", "BALED", true),
            ("xalek_yyyy", "SALEK", false),
            ("xalek_yyyy", "ALIKE", true),
            ("xalek_yyyy", "ANKLE", true),
            ("xalek_yyyy", "FLAKE", true),
            ("xalek_yyyy", "LEAKY", true),
            ("xalek_yyyy", "SLAKE", true),
            ("xkela_yyyy", "SALEK", true),
            ("xkela_yyyy", "ALIKE", true),
            ("xkela_yyyy", "FLAKE", true),
            ("xkela_yyyy", "LEAKY", true),
            ("xkela_yyyy", "SLAKE", true),
            ("xkela_yyyy", "LATKE", true),
            ("Alekxgyyy", "ANKLE", true),
            ("abackyy", "BREAD", true),
            ("abacKyy__g", "BREAK", true),
            ("abackyy__", "BREAM", true),
            ("abacKyy__g", "ABACK", false),
            ("abacK__yyg", "ABACK", false),
            ("abamkyy", "BREAD", true),
            ("abamKyy__g", "BREAK", true),
            ("abamkyy_y", "BREAM", true),
            ("BREAd____y", "BREAD", false),
            ("BREAdgggg", "BREAK", true),
            ("BREAdgggg", "BREAM", true),
            ("WhoSEg__gg", "WORSE", false),
            ("WhoSEg_ygg", "WORSE", true),
            ("WorSEg__gg", "WHOSE", false),
            ("WorSEgy_gg", "WHOSE", true),
            ("pooli_yy", "POOLS", false),
            ("oxxoxy__y", "POOLS", true),
            ("oxoxxy_y", "POOLS", false),
            ("OXOXXy_g", "POOLS", true),
            ("OXXOOy__yy", "POOLS", false),
        ];

        for (filter_str, candidate, expected) in test_cases {
            let (guess, word_clue) = WordClue::new_from_guess_n_clue_str(filter_str);
            eprintln!("Filter: {filter_str}, Candidate: {candidate}, Expected: {expected}");
            assert_eq!(
                word_clue.is_match(guess, candidate),
                expected,
                "Filter {filter_str}"
            );
        }
    }
}
