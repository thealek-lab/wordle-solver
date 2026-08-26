use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::filter::Filter;
use crate::guess_set::{Guess, GuessSet};

pub const DEFAULT_SOLUTIONS_FILE: &str = "./solutions.txt";
pub const DEFAULT_GUESSES_FILE: &str = "./all_guesses_2022_11K.txt";

pub fn resolve_default_path(file_name: &str) -> String {
    if Path::new(file_name).exists() {
        file_name.to_owned()
    } else {
        format!("../{}", file_name.trim_start_matches("./"))
    }
}

pub struct Solver {
    pub verbosity: u32,
    pub all_solutions: Vec<String>,
    pub filtered_sols: Vec<String>,
    pub all_guesses: Vec<String>,
    all_solution_set: HashSet<String>,
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
        ensure_unique(&all_solutions, "solution", solution_file_name)?;
        let all_solutions = sorted(all_solutions);

        let mut all_guesses = if guesses_file_name.is_empty() {
            all_solutions.clone()
        } else {
            read_words(guesses_file_name, false)?
        };
        ensure_unique(&all_guesses, "guess", guesses_file_name)?;
        all_guesses.extend(all_solutions.iter().cloned());

        for solution in &all_solutions {
            if !all_guesses.contains(solution) {
                return Err(format!(
                    "Missing solution {solution} in guesses file: {guesses_file_name}"
                ));
            }
        }

        let all_guesses = sorted(all_guesses);
        if verbosity >= 1 {
            println!("All Possible Solutions: {}", all_solutions.len());
            println!("All Possible Guesses: {}", all_guesses.len());
        }

        Ok(Self {
            verbosity,
            filtered_sols: all_solutions.clone(),
            all_solution_set: all_solutions.iter().cloned().collect(),
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

    pub fn filter(&mut self, filter_str_list: &[String]) -> Result<(), String> {
        for filter_str in filter_str_list {
            let filter = Filter::make_filter(filter_str)?;
            let matches = filter.make_filter_func();
            self.filtered_sols.retain(|solution| matches(solution));
        }
        self.filtered_sols.sort();
        Ok(())
    }

    pub fn try_guess(
        &self,
        guess_str: &str,
        lowest_remaining_cnt: f64,
    ) -> Result<GuessSet, String> {
        let mut partitions: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut hints = Vec::with_capacity(self.filtered_sols.len());

        for maybe_sol in &self.filtered_sols {
            let hint = Filter::make_hint(guess_str, maybe_sol)?;
            if !hint.is_match(guess_str) {
                partitions
                    .entry(hint.to_string())
                    .or_default()
                    .push(maybe_sol.clone());
            }
            hints.push((maybe_sol.clone(), hint));
        }

        let mut guess_set =
            GuessSet::new(guess_str.to_owned(), self.filtered_sols.len(), true, false);
        for (maybe_sol, hint) in hints {
            let remaining = if hint.is_match(guess_str) {
                Vec::new()
            } else {
                partitions
                    .get(&hint.to_string())
                    .cloned()
                    .unwrap_or_default()
            };
            let guess = Guess::new(maybe_sol.clone(), hint.clone(), remaining);
            if self.verbosity >= 2 {
                println!(
                    "Guess {guess_str} number remaining: {} with solution {maybe_sol} hint {hint}",
                    guess.remaining_cnt
                );
            }
            guess_set.add_guess(guess);

            if guess_set.remaining_cnt > lowest_remaining_cnt {
                break;
            }
        }
        Ok(guess_set)
    }

    fn score_guess(&self, guess_str: &str) -> Result<f64, String> {
        let guess = word_bytes(guess_str)?;
        let mut counts = [0usize; 243];

        for maybe_sol in &self.filtered_sols {
            let pattern = feedback_key(&guess, maybe_sol.as_bytes());
            if pattern != 242 {
                counts[pattern as usize] += 1;
            }
        }

        Ok(counts
            .iter()
            .map(|count| {
                let count = *count as f64;
                count * (0.5 + count / 2.0)
            })
            .sum())
    }

    pub fn find_best_guess(
        &self,
        hint_best: Option<GuessSet>,
        hard_mode: bool,
        reverse: bool,
    ) -> Result<GuessSet, String> {
        let start_time = Instant::now();
        let num_sols = self.filtered_sols.len();
        if num_sols == 0 {
            return Err("No solutions left to guess from".to_owned());
        }

        let mut guess_list = if hard_mode {
            self.filtered_sols.clone()
        } else {
            self.all_guesses.clone()
        };
        if reverse {
            guess_list.reverse();
        }
        let total_guesses = guess_list.len();

        let mut best_guess = match hint_best {
            Some(guess) => guess,
            None => {
                let mut guess = self.try_guess(&self.filtered_sols[0], f64::MAX)?;
                guess.is_in_remaining_sol_set = true;
                if self.verbosity >= 1 {
                    println!(
                        "Initial guess {}: expected solutions remaining {:.2} on average from {}/{}",
                        guess, guess.remaining_avg, guess.remaining_cnt, num_sols
                    );
                }
                guess
            }
        };
        let remaining_set: HashSet<_> = self.filtered_sols.iter().collect();
        let mut last_report = start_time;

        for (index, guess_str) in guess_list.iter().enumerate() {
            let score = self.score_guess(guess_str)?;
            let mut new_guess = GuessSet::new(
                guess_str.clone(),
                num_sols,
                self.all_solution_set.contains(guess_str),
                remaining_set.contains(guess_str),
            );
            new_guess.remaining_cnt = score;
            new_guess.remaining_avg = score / num_sols as f64;

            let elapsed = start_time.elapsed().as_secs_f64();
            let percent_complete = 100.0 * (index + 1) as f64 / total_guesses as f64;
            let time_remaining = elapsed * (total_guesses - index - 1) as f64 / (index + 1) as f64;

            if new_guess.is_better_than(&best_guess, self.verbosity) {
                best_guess = self.try_guess(guess_str, score)?;
                best_guess.is_in_remaining_sol_set = remaining_set.contains(guess_str);
                best_guess.is_in_tot_sol_set = self.all_solution_set.contains(guess_str);
                if self.verbosity >= 1 {
                    println!(
                        "[{percent_complete:.2}% after {elapsed:.2}s done in {time_remaining:.2}s] Best guess {}: expected solutions remaining {:.2} on average from {}/{}",
                        best_guess, best_guess.remaining_avg, best_guess.remaining_cnt, num_sols
                    );
                }
                last_report = Instant::now();
            } else if self.verbosity >= 1 && last_report.elapsed().as_secs_f64() >= 5.0 {
                println!(
                    "[{percent_complete:.2}% after {elapsed:.2}s done in {time_remaining:.2}s] Best guess is still {} {:.2} (last guess was {} {:.2}+)",
                    best_guess, best_guess.remaining_avg, guess_str, new_guess.remaining_avg
                );
                last_report = Instant::now();
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        if self.verbosity >= 1 {
            println!("Time taken: {elapsed:.2} seconds");
            if !self.all_solutions.contains(&best_guess.guess_str) {
                println!(
                    "WARNING: Best guess {} is not in the solutions file!",
                    best_guess
                );
            }
            println!(
                "Best guess {}: expected solutions remaining {:.2} on average from {}/{}",
                best_guess, best_guess.remaining_avg, best_guess.remaining_cnt, num_sols
            );
        }

        Ok(best_guess)
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

fn ensure_unique(words: &[String], kind: &str, file_name: &str) -> Result<(), String> {
    if words.iter().collect::<HashSet<_>>().len() != words.len() {
        return Err(format!("Duplicate {kind} in {file_name}"));
    }
    Ok(())
}

fn sorted(mut words: Vec<String>) -> Vec<String> {
    words.sort();
    words
}

fn word_bytes(word: &str) -> Result<[u8; 5], String> {
    let bytes = word.as_bytes();
    if bytes.len() != 5 || !bytes.iter().all(u8::is_ascii_alphabetic) {
        return Err(format!("Word '{word}' is not a 5-letter alphabetic word"));
    }
    Ok([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]])
}

fn feedback_key(guess: &[u8; 5], solution: &[u8]) -> u16 {
    let mut pattern = [0u8; 5];
    let mut remaining = [true; 5];

    for index in 0..5 {
        if guess[index] == solution[index] {
            pattern[index] = 2;
            remaining[index] = false;
        }
    }

    for index in 0..5 {
        if pattern[index] == 0 {
            if let Some(position) =
                (0..5).find(|&position| remaining[position] && solution[position] == guess[index])
            {
                pattern[index] = 1;
                remaining[position] = false;
            }
        }
    }

    pattern
        .iter()
        .fold(0u16, |key, &value| key * 3 + value as u16)
}
