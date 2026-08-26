use std::fs::File;
use std::io::Write;
use std::time::Instant;

use crate::filter::Filter;
use crate::solver::{DEFAULT_GUESSES_FILE, DEFAULT_SOLUTIONS_FILE, Solver, resolve_default_path};

pub const BEST_INITIAL_GUESS: &str = "ROATE";

pub struct GameSim {
    solution: String,
    verbosity: u32,
}

impl GameSim {
    pub fn new(solution: &str, verbosity: u32) -> Self {
        if verbosity >= 2 {
            println!("Creating GameSim with hidden solution {solution}");
        }
        Self {
            solution: solution.to_uppercase(),
            verbosity,
        }
    }

    pub fn run(&self, initial_guess: &str) -> Result<Vec<(String, String)>, String> {
        let start_time = Instant::now();
        let solutions_file = resolve_default_path(DEFAULT_SOLUTIONS_FILE);
        let guesses_file = resolve_default_path(DEFAULT_GUESSES_FILE);
        let mut sols = Solver::new(&solutions_file, &guesses_file, self.verbosity)?;
        if !sols.all_solutions.contains(&self.solution) {
            return Err(format!(
                "Invalid solution {} is not in the official list!",
                self.solution
            ));
        }

        let initial_guess = initial_guess.to_uppercase();
        let mut hints = vec![Filter::make_hint(&initial_guess, &self.solution)?.to_string()];
        sols.filter(&hints)?;
        let mut guess_list = vec![(initial_guess.clone(), sols.len().to_string())];
        println!(
            "After first guess {initial_guess} Solutions: {}",
            sols.len()
        );

        while !sols.is_empty() {
            let best_guess = sols.find_best_guess(None, false, false)?;
            let hint = Filter::make_hint(&best_guess.guess_str, &self.solution)?.to_string();
            hints.push(hint);
            sols.filter(&hints)?;
            guess_list.push((best_guess.guess_str, sols.len().to_string()));

            println!(
                "After guess {} {} Solutions: {}",
                guess_list.len(),
                guess_list.last().unwrap().0,
                sols.len()
            );
            if !sols.is_empty() && (self.verbosity >= 2 || sols.len() <= 10) {
                println!("   {:?}", sols.filtered_sols);
            }
            if sols.len() == 1 {
                let single_sol = sols.filtered_sols[0].clone();
                if guess_list.last().unwrap().0 != single_sol {
                    guess_list.push((single_sol, "*".to_owned()));
                }
                break;
            }
            if guess_list.len() >= 7 {
                break;
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let found = guess_list
            .last()
            .map(|guess| guess.0 == self.solution)
            .unwrap_or(false);
        if found {
            println!(
                "Found solution {:?} in {} steps after {elapsed:.2}s!",
                guess_list.last().unwrap(),
                guess_list.len()
            );
            Ok(guess_list)
        } else {
            Err(format!(
                "Cannot find solution {}: best guess {:?}!",
                self.solution,
                guess_list.last()
            ))
        }
    }

    pub fn calculate_initial_guess_performance(
        initial_guess: &str,
        verbosity: u32,
    ) -> Result<(), String> {
        let initial_guess = initial_guess.to_uppercase();
        let solutions_file = resolve_default_path(DEFAULT_SOLUTIONS_FILE);
        let guesses_file = resolve_default_path(DEFAULT_GUESSES_FILE);
        let sols = Solver::new(&solutions_file, &guesses_file, verbosity)?;
        if !sols.all_guesses.contains(&initial_guess) {
            return Err(format!(
                "Initial guess '{initial_guess}' is not in guess list!"
            ));
        }

        let file_name = format!("wordle_initial_guess_{initial_guess}_results.txt");
        let mut file = File::create(&file_name)
            .map_err(|error| format!("Could not create {file_name}: {error}"))?;
        let total = sols.all_solutions.len();
        let start_time = Instant::now();
        let mut total_steps = 0;

        for (index, solution) in sols.all_solutions.iter().enumerate() {
            println!("Testing solution {solution}");
            let sim = Self::new(solution, verbosity);
            let guesses = sim.run(&initial_guess)?;
            total_steps += guesses.len();
            let mut line = format!("{solution}, {}", guesses.len());
            for (guess, count) in guesses {
                line.push_str(&format!(", {guess}({count})"));
            }
            writeln!(file, "{line}")
                .map_err(|error| format!("Could not write {file_name}: {error}"))?;
            file.flush()
                .map_err(|error| format!("Could not flush {file_name}: {error}"))?;

            let elapsed = start_time.elapsed().as_secs_f64();
            let average = total_steps as f64 / (index + 1) as f64;
            let percent = 100.0 * (index + 1) as f64 / total as f64;
            let remaining = elapsed * (total - index - 1) as f64 / (index + 1) as f64;
            println!(
                "[{percent:.2}% after {elapsed:.2}s done in {remaining:.2}s] avg={average:.2}={total_steps}/{} ",
                index + 1
            );
        }
        Ok(())
    }
}
