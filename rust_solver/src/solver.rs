use std::path::Path;
use std::{fs, time::Instant};

use crate::filter::{WORD_LENGTH, WordChars, WordClue};

pub const DEFAULT_SOLUTIONS_FILE: &str = "./solutions.txt";
pub const DEFAULT_GUESSES_FILE: &str = "./all_guesses_2022_11K.txt";

pub fn resolve_default_path(file_name: &str) -> String {
    if Path::new(file_name).exists() {
        file_name.to_owned()
    } else {
        format!("../{}", file_name.trim_start_matches("./"))
    }
}

#[derive(Clone)]
pub struct GuessScore {
    pub entropy: f64,
    pub word_chars: WordChars,
    pub in_sols: bool,
}

impl GuessScore {
    pub fn new(word_chars: WordChars, entropy: f64, in_sols: bool) -> Self {
        Self {
            word_chars,
            entropy,
            in_sols,
        }
    }

    pub fn guess_str(&self) -> String {
        let mut out = self.word_chars.iter().collect();

        if self.in_sols {
            out += "+";
        }

        out
    }
}

pub struct Solver {
    pub verbosity: u32,
    pub all_solutions: Vec<WordChars>,
    pub filtered_sols: Vec<WordChars>,
    pub all_guesses: Vec<WordChars>,
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

        let mut all_solutions = read_words(solution_file_name, true)?;
        all_solutions.sort();
        all_solutions.dedup();

        let mut all_guesses = if guesses_file_name.is_empty() {
            vec![]
        } else {
            read_words(guesses_file_name, false)?
        };
        all_guesses.extend(all_solutions.iter().cloned());
        all_guesses.sort();
        all_guesses.dedup();

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
            .collect::<Vec<_>>();

        let all_guesses = all_guesses
            .iter()
            .map(|guess| WordClue::make_word_chars(guess))
            .collect::<Vec<_>>();

        Ok(Self {
            verbosity,
            filtered_sols: all_solutions.clone(),
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
        self.filtered_sols.sort();
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
        let entropy = buckets.into_iter().fold(0.0, |total, size| {
            if size > 0 {
                let probability = size as f64 / num_sols as f64;
                total - probability * probability.log2()
            } else {
                total
            }
        });

        assert!(
            entropy >= 0.0,
            "Entropy should be non-negative, but got {entropy}"
        );
        assert!(
            entropy.is_finite(),
            "Entropy should be finite, but got {entropy}"
        );

        entropy
    }

    pub fn find_best_guess(
        &self,
        hard_mode: bool,
        reverse: bool,
        max_num: usize,
    ) -> Result<Vec<GuessScore>, String> {
        let num_sols = self.filtered_sols.len();
        if num_sols == 0 {
            return Err("No solutions left to guess from".to_owned());
        }

        if num_sols == 1 {
            return Ok(vec![GuessScore::new(self.filtered_sols[0], 100.0, true)]);
        }

        let mut guess_list = if hard_mode {
            &self.filtered_sols
        } else {
            &self.all_guesses
        };

        let mut reverse_list = vec![];
        if reverse {
            reverse_list = guess_list.iter().rev().copied().collect();
            guess_list = &reverse_list;
        }

        let mut best_list = vec![GuessScore::new(['_'; WORD_LENGTH], 0.0, false)];
        for guess_chars in guess_list {
            let entropy = self.calc_guess_entropy(*guess_chars);

            let list_len = best_list.len();
            let worst_item = &best_list[list_len - 1];

            if entropy > worst_item.entropy {
                best_list.push(GuessScore::new(
                    *guess_chars,
                    entropy,
                    self.filtered_sols.binary_search(guess_chars).is_ok(),
                ));

                best_list.sort_by(|a, b| {
                    b.entropy
                        .total_cmp(&a.entropy)
                        .then_with(|| b.in_sols.cmp(&a.in_sols))
                });
                best_list.truncate(max_num);
            } else if entropy == worst_item.entropy
                && !worst_item.in_sols
                && self.filtered_sols.binary_search(guess_chars).is_ok()
            {
                best_list.push(GuessScore::new(*guess_chars, entropy, true));
                best_list.sort_by(|a, b| {
                    b.entropy
                        .total_cmp(&a.entropy)
                        .then_with(|| b.in_sols.cmp(&a.in_sols))
                });
                best_list.truncate(max_num);
            }
        }

        Ok(best_list)
    }

    pub fn simulate(
        &mut self,
        initial_guess_str: &str,
        solution_str: &str,
        reverse: bool,
    ) -> Result<Vec<(GuessScore, usize)>, String> {
        let start_time = Instant::now();

        let solution = WordClue::make_word_chars(solution_str);
        let sol_chars = solution;

        let initial_guess = WordClue::make_word_chars(initial_guess_str);

        self.filtered_sols = self.all_solutions.clone();

        self.filter_by_guess_n_solution(initial_guess, sol_chars)?;
        let mut guesses = vec![(GuessScore::new(initial_guess, 0.0, false), self.len())];
        if self.verbosity >= 1 {
            println!(
                "After first guess {initial_guess_str} Solutions: {}",
                self.len()
            );
        }

        while self.len() > 1 {
            let ranked_guesses = self.find_best_guess(false, reverse, 1)?;
            let score = &ranked_guesses[0];
            self.filter_by_guess_n_solution(score.word_chars, sol_chars)?;

            guesses.push((score.clone(), self.len()));
            if self.verbosity >= 1 {
                println!(
                    "After guess {} {} Solutions: {}",
                    guesses.len(),
                    score.guess_str(),
                    self.len()
                );
            }
            if self.verbosity >= 2 && !self.is_empty() && self.len() <= 10 {
                println!(
                    "   {}",
                    self.filtered_sols
                        .iter()
                        .map(|a| a.iter().collect::<String>())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if self.len() == 1 {
                if guesses.last().unwrap().0.word_chars != score.word_chars {
                    guesses.push((GuessScore::new(score.word_chars, 100.0, true), 0));
                }

                break;
            }
            if guesses.len() >= 7 {
                break;
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let last_guess = guesses.last();
        if let Some((best_guess, _)) = last_guess {
            let solution = self.filtered_sols[0];
            if best_guess.word_chars != solution {
                guesses.push((GuessScore::new(solution, 100.0, true), 0));
            }

            println!(
                "Found solution {solution_str} in {} steps after {elapsed:.2}s!",
                guesses.len()
            );
            Ok(guesses)
        } else {
            Err(format!(
                "Cannot find solution {solution_str}: best guess {}!",
                last_guess.unwrap().0.guess_str()
            ))
        }
    }
}

fn read_words(file_name: &str, solution_file: bool) -> Result<Vec<String>, String> {
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
        .collect::<Vec<_>>();

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
