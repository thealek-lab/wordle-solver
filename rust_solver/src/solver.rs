use std::collections::{BTreeSet, BinaryHeap, HashSet};
use std::fs;
use std::path::Path;

use crate::filter::{WordChars, WordClue};

pub const DEFAULT_SOLUTIONS_FILE: &str = "./solutions.txt";
pub const DEFAULT_GUESSES_FILE: &str = "./all_guesses_2022_11K.txt";

pub fn resolve_default_path(file_name: &str) -> String {
    if Path::new(file_name).exists() {
        file_name.to_owned()
    } else {
        format!("../{}", file_name.trim_start_matches("./"))
    }
}

pub struct GuessScore {
    pub entropy: f64,
    pub is_solution: bool,
    pub guess_chars: WordChars,
}

impl GuessScore {
    pub fn new(entropy: f64, is_solution: bool, guess_chars: WordChars) -> Self {
        Self {
            entropy,
            is_solution,
            guess_chars,
        }
    }

    pub fn guess_str(&self) -> String {
        let mut guess_str = self.guess_chars.iter().collect();
        if self.is_solution {
            guess_str += "+";
        }

        guess_str
    }
}

impl PartialEq for GuessScore {
    fn eq(&self, other: &Self) -> bool {
        self.entropy == other.entropy
            && self.is_solution == other.is_solution
            && self.guess_chars == other.guess_chars
    }
}

impl Eq for GuessScore {}

impl PartialOrd for GuessScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GuessScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        assert!(self.entropy.is_finite() && other.entropy.is_finite());
        assert!(self.entropy >= 0.0 && other.entropy >= 0.0);

        // Sort by entropy first
        let entropy_cmp = self.entropy.total_cmp(&other.entropy);
        if entropy_cmp != std::cmp::Ordering::Equal {
            return entropy_cmp;
        }

        let is_sol_cmp = self.is_solution.cmp(&other.is_solution);
        if is_sol_cmp != std::cmp::Ordering::Equal {
            return is_sol_cmp;
        }

        // Sort alphabetically if all else fails
        other.guess_chars.cmp(&self.guess_chars)
    }
}

pub struct Solver {
    pub verbosity: u32,
    pub all_solutions: BTreeSet<WordChars>,
    pub filtered_sols: BTreeSet<WordChars>,
    pub all_guesses: BTreeSet<WordChars>,
}

impl Solver {
    pub fn new(
        solution_file_name: &str,
        guesses_file_name: &str,
        verbosity: u32,
    ) -> Result<Self, String> {
        if verbosity >= 2 {
            println!("Creating Solver with guesses {guesses_file_name}");
        }

        let all_solutions = read_words(solution_file_name, true)?;

        let mut all_guesses = if guesses_file_name.is_empty() {
            HashSet::new()
        } else {
            read_words(guesses_file_name, false)?
        };
        all_guesses.extend(all_solutions.iter().cloned());

        for solution in &all_solutions {
            if !all_guesses.contains(solution) {
                return Err(format!(
                    "Missing solution {solution} in guesses file: {guesses_file_name}"
                ));
            }
        }

        if verbosity >= 1 {
            println!("All Possible Solutions: {}", all_solutions.len());
            println!("All Possible Guesses: {}", all_guesses.len());
        }

        let all_solutions = all_solutions
            .iter()
            .map(|solution| WordClue::make_word_chars(solution))
            .collect::<BTreeSet<_>>();

        let all_guesses = all_guesses
            .iter()
            .map(|guess| WordClue::make_word_chars(guess))
            .collect::<BTreeSet<_>>();

        Ok(Self {
            verbosity,
            filtered_sols: all_solutions.clone().into_iter().collect(),
            all_solutions,
            all_guesses,
        })
    }

    pub fn len(&self) -> usize {
        self.filtered_sols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.filtered_sols.is_empty()
    }

    pub fn filter_by_guess_n_clue_list(
        &mut self,
        guess_n_clue_list: &[String],
    ) -> Result<(), String> {
        for guess_n_clue_str in guess_n_clue_list {
            let (guess_chars, word_clue) = WordClue::new_from_guess_n_clue_str(guess_n_clue_str);
            self.filtered_sols
                .retain(|maybe_sol: &WordChars| word_clue.is_match(guess_chars, *maybe_sol));
        }
        Ok(())
    }

    pub fn filter_by_guess_n_solution(
        &mut self,
        guess_chars: WordChars,
        sol_chars: WordChars,
    ) -> Result<(), String> {
        let sol_clue = WordClue::calc_val_from_guess_n_solution(guess_chars, sol_chars.as_slice());
        self.filtered_sols.retain(|maybe_sol: &WordChars| {
            WordClue::calc_val_from_guess_n_solution(guess_chars, maybe_sol) == sol_clue
        });

        Ok(())
    }

    pub fn calc_guess_entropy(&self, guess_chars: WordChars) -> f64 {
        let num_sols = self.filtered_sols.len();
        let mut buckets = [0usize; 243];
        for answer in &self.filtered_sols {
            let feedback = WordClue::calc_val_from_guess_n_solution(guess_chars, answer);
            buckets[feedback] += 1;
        }
        buckets.into_iter().fold(0.0, |total, size| {
            if size > 0 {
                let probability = size as f64 / num_sols as f64;
                total - probability * probability.log2()
            } else {
                total
            }
        })
    }

    pub fn find_best_guesses(
        &self,
        hard_mode: bool,
        reverse: bool,
        max_results: usize,
    ) -> Result<Vec<GuessScore>, String> {
        let num_sols = self.filtered_sols.len();
        if num_sols == 0 {
            return Err("No solutions left to guess from".to_owned());
        }

        if max_results == 0 {
            return Ok(vec![]);
        }

        if num_sols == 1 {
            return Ok(vec![
                (GuessScore {
                    entropy: 100.0,
                    is_solution: true,
                    guess_chars: self.filtered_sols.iter().next().unwrap().clone(),
                }),
            ]);
        }

        let mut guess_list = if hard_mode {
            &self.filtered_sols
        } else {
            &self.all_guesses
        };

        let mut reverse_list: Vec<WordChars> = vec![];
        if reverse {
            // TBF TODO reverse_list.extend(guess_list.iter().rev());
            // guess_list = &reverse_list;
        }

        let mut ranked_guesses = BinaryHeap::new();
        for guess in guess_list {
            ranked_guesses.push(GuessScore {
                entropy: self.calc_guess_entropy(*guess),
                is_solution: self.filtered_sols.contains(guess),
                guess_chars: *guess,
            });
        }

        let mut top_guesses: Vec<GuessScore> =
            ranked_guesses.into_iter().take(max_results).collect();

        top_guesses.sort();
        top_guesses.reverse();

        Ok(top_guesses)
    }
}

fn read_words(file_name: &str, solution_file: bool) -> Result<HashSet<String>, String> {
    let contents = fs::read_to_string(file_name)
        .map_err(|error| format!("Could not read {file_name}: {error}"))?;
    let words = contents
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            let valid = if solution_file {
                fields.len() == 2 && fields[0] != "Word"
            } else {
                fields.len() == 1
            };
            valid.then(|| fields[0].to_uppercase())
        })
        .collect::<HashSet<_>>();

    for word in &words {
        if word.chars().count() != 5 {
            return Err(format!(
                "Invalid {} length: {word}",
                if solution_file { "solution" } else { "guess" }
            ));
        }
    }
    Ok(words)
}
