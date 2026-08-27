use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::filter::Filter;

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
            vec![]
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

    pub fn find_best_guess(
        &self,
        hard_mode: bool,
        reverse: bool,
    ) -> Result<Vec<(String, f64)>, String> {
        let num_sols = self.filtered_sols.len();
        if num_sols == 0 {
            return Err("No solutions left to guess from".to_owned());
        }

        if num_sols == 1 {
            return Ok(vec![(self.filtered_sols[0].clone(), 0.0)]);
        }

        let guess_list = if hard_mode {
            &self.filtered_sols
        } else {
            &self.all_guesses
        };
        if reverse {
            //TBD guess_list.reverse();
        }

        let mut best_entropy = 0.0;
        let mut best_guess = "<None>".to_string();
        for guess in guess_list {
            let guess_bytes = guess
                .as_bytes()
                .try_into()
                .expect("Guess is not 5 characters long");
            let mut buckets = [0usize; 243];
            for answer in &self.filtered_sols {
                let feedback = feedback_key(guess_bytes, answer.as_bytes());
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

            if entropy > best_entropy {
                best_entropy = entropy;
                best_guess = guess.clone();
            }
        }

        Ok(vec![(best_guess, best_entropy)])
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

fn feedback_key(guess: &[u8; 5], solution: &[u8]) -> usize {
    let mut pattern = [0usize; 5];
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

    pattern.iter().fold(0, |key, &value| key * 3 + value)
}
