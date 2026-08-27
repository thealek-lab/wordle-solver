use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::Instant;

use crate::filter::Filter;
use crate::solver::{DEFAULT_SOLUTIONS_FILE, Solver, resolve_default_path};

pub const BEST_INITIAL_GUESS: &str = "SLATE";

pub struct GameSim {
    solution: String,
    verbosity: u32,
    solver: Solver,
}

impl GameSim {
    pub fn new(solution: &str, guesses_file: &str, verbosity: u32) -> Result<Self, String> {
        if verbosity >= 2 {
            println!("Creating GameSim with hidden solution {solution}");
        }

        let solution = solution.to_uppercase();

        let solver = Solver::new(
            &resolve_default_path(DEFAULT_SOLUTIONS_FILE),
            &guesses_file,
            verbosity,
        )?;
        if !solver.all_solutions.contains(&solution) {
            return Err(format!(
                "Invalid solution {solution} is not in the official list!"
            ));
        }

        Ok(Self {
            solution,
            verbosity,
            solver,
        })
    }

    pub fn run(&mut self, initial_guess: &str) -> Result<Vec<(String, String)>, String> {
        let start_time = Instant::now();

        self.solver.filtered_sols = self.solver.all_solutions.clone();

        let initial_guess = initial_guess.to_uppercase();
        let mut hints = vec![Filter::make_hint(&initial_guess, &self.solution)?.to_string()];
        self.solver.filter(&hints)?;
        let mut guesses = vec![(initial_guess.clone(), self.solver.len().to_string())];
        println!(
            "After first guess {initial_guess} Solutions: {}",
            self.solver.len()
        );
        while !self.solver.is_empty() {
            let ranked_guesses = self.solver.find_best_guess(false, false)?;
            let best = &ranked_guesses[0];
            hints.push(Filter::make_hint(&best.0, &self.solution)?.to_string());
            self.solver.filter(&hints)?;
            guesses.push((best.0.clone(), self.solver.len().to_string()));
            println!(
                "After guess {} {} Solutions: {}",
                guesses.len(),
                guesses.last().unwrap().0,
                self.solver.len()
            );
            if !self.solver.is_empty() && (self.verbosity >= 2 || self.solver.len() <= 10) {
                println!("   {:?}", self.solver.filtered_sols);
            }
            if self.solver.len() == 1 {
                let solution = self.solver.filtered_sols[0].clone();
                if guesses.last().unwrap().0 != solution {
                    guesses.push((solution, "*".to_owned()));
                }
                break;
            }
            if guesses.len() >= 7 {
                break;
            }
        }
        let elapsed = start_time.elapsed().as_secs_f64();
        if guesses
            .last()
            .map(|guess| guess.0 == self.solution)
            .unwrap_or(false)
        {
            println!(
                "Found solution {:?} in {} steps after {elapsed:.2}s!",
                guesses.last().unwrap(),
                guesses.len()
            );
            Ok(guesses)
        } else {
            Err(format!(
                "Cannot find solution {}: best guess {:?}!",
                self.solution,
                guesses.last()
            ))
        }
    }

    pub fn calculate_initial_guess_performance_from(
        &mut self,
        initial_guess: &str,
        resume_file: Option<&str>,
    ) -> Result<(), String> {
        let initial_guess = initial_guess.to_uppercase();

        if !self.solver.all_guesses.contains(&initial_guess) {
            return Err(format!(
                "Initial guess '{initial_guess}' is not in guess list!"
            ));
        }
        let file_name = resume_file
            .map(str::to_owned)
            .unwrap_or_else(|| format!("wordle_initial_guess_{initial_guess}_results.txt"));
        let (completed, mut total_steps) = match resume_file {
            Some(path) => read_completed_results(path, &self.solver.all_solutions)?,
            None => (HashSet::new(), 0),
        };
        let mut output = if resume_file.is_some() {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_name)
        } else {
            File::create(&file_name)
        }
        .map_err(|error| format!("Could not open {file_name}: {error}"))?;
        let total = self.solver.all_solutions.len();
        let start_time = Instant::now();
        let mut completed_count = completed.len();
        for solution in &self.solver.all_solutions.clone() {
            if completed.contains(solution) {
                continue;
            }
            println!("Testing solution {solution}");
            self.solution = solution.clone();

            let result = self.run(&initial_guess)?;
            total_steps += result.len();
            let mut line = format!("{solution}, {}", result.len());
            for (guess, count) in result {
                line.push_str(&format!(", {guess}({count})"));
            }
            writeln!(output, "{line}")
                .map_err(|error| format!("Could not write {file_name}: {error}"))?;
            output
                .flush()
                .map_err(|error| format!("Could not flush {file_name}: {error}"))?;
            completed_count += 1;
            let elapsed = start_time.elapsed().as_secs_f64();
            let average = total_steps as f64 / completed_count as f64;
            let percent = 100.0 * completed_count as f64 / total as f64;
            let remaining = elapsed * (total - completed_count) as f64 / completed_count as f64;
            println!(
                "[{percent:.2}% after {elapsed:.2}s done in {remaining:.2}s] avg={average:.2}={total_steps}/{completed_count} "
            );
        }
        Ok(())
    }
}

fn read_completed_results(
    file_name: &str,
    all_solutions: &[String],
) -> Result<(HashSet<String>, usize), String> {
    let contents = std::fs::read_to_string(file_name)
        .map_err(|error| format!("Could not read resume file {file_name}: {error}"))?;
    let solution_set: HashSet<_> = all_solutions.iter().collect();
    let mut completed = HashSet::new();
    let mut total_steps = 0;
    for (line_number, line) in contents.lines().enumerate() {
        let fields: Vec<_> = line.split(',').map(str::trim).collect();
        if fields.len() < 2 {
            return Err(format!(
                "Invalid resume row {} in {file_name}",
                line_number + 1
            ));
        }
        let solution = fields[0].to_uppercase();
        if !solution_set.contains(&solution) {
            return Err(format!("Unknown solution '{solution}' in {file_name}"));
        }
        if !completed.insert(solution.clone()) {
            return Err(format!("Duplicate solution '{solution}' in {file_name}"));
        }
        total_steps += fields[1].parse::<usize>().map_err(|error| {
            format!(
                "Invalid step count in resume row {}: {error}",
                line_number + 1
            )
        })?;
    }
    Ok((completed, total_steps))
}
