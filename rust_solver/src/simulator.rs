use std::collections::{BTreeSet, HashSet};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::Instant;

use crate::filter::{WordChars, WordClue};
use crate::solver::{DEFAULT_SOLUTIONS_FILE, GuessScore, Solver, resolve_default_path};

pub const BEST_INITIAL_GUESS: &str = "SLATE";

pub struct GameSim {
    solution: GuessScore,
    verbosity: u32,
    solver: Solver,
}

impl GameSim {
    pub fn new(solution_str: &str, guesses_file: &str, verbosity: u32) -> Result<Self, String> {
        if verbosity >= 2 {
            println!("Creating GameSim with hidden solution {solution_str}");
        }

        let solution = GuessScore::new(0.0, false, WordClue::make_word_chars(solution_str));

        let solver = Solver::new(
            &resolve_default_path(DEFAULT_SOLUTIONS_FILE),
            guesses_file,
            verbosity,
        )?;
        if !solver.all_solutions.contains(&solution.guess_chars) {
            return Err(format!(
                "Invalid solution {solution_str} is not in the official list!"
            ));
        }

        Ok(Self {
            solution,
            verbosity,
            solver,
        })
    }

    pub fn run(&mut self, initial_guess_str: &str) -> Result<Vec<(GuessScore, String)>, String> {
        let start_time = Instant::now();

        let initial_guess = WordClue::make_word_chars(initial_guess_str);

        self.solver.filtered_sols = self.solver.all_solutions.clone();

        self.solver
            .filter_by_guess_n_solution(initial_guess, self.solution.guess_chars)?;

        let initial_score = GuessScore::new(0.0, false, initial_guess);

        let mut guesses = vec![(initial_score, self.solver.len().to_string())];
        if self.verbosity >= 1 {
            println!(
                "After first guess {initial_guess_str} Solutions: {}",
                self.solver.len()
            );
        }
        while !self.solver.is_empty() {
            let mut ranked_guesses = self.solver.find_best_guesses(false, false, 1)?;
            let best_score = ranked_guesses.remove(0);
            let best_guess_str = best_score.guess_str();
            self.solver
                .filter_by_guess_n_solution(best_score.guess_chars, self.solution.guess_chars)?;
            guesses.push((best_score, self.solver.len().to_string()));
            if self.verbosity >= 1 {
                println!(
                    "After guess {} {best_guess_str} Solutions: {}",
                    guesses.len(),
                    self.solver.len()
                );
            }
            if self.verbosity >= 2 && !self.solver.is_empty() && self.solver.len() <= 10 {
                println!(
                    "   {}",
                    self.solver
                        .filtered_sols
                        .iter()
                        .map(|w| w.iter().collect::<String>())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if self.solver.len() == 1 {
                let solution = self.solver.filtered_sols.iter().next().unwrap();
                if guesses.last().unwrap().0.guess_chars != *solution {
                    let final_score = GuessScore::new(100.0, true, *solution);
                    guesses.push((final_score, "*".to_owned()));
                }
                break;
            }
            if guesses.len() >= 7 {
                break;
            }
        }
        let elapsed = start_time.elapsed().as_secs_f64();
        let (best_score, _) = guesses.last().unwrap();
        if best_score.guess_chars == self.solution.guess_chars {
            println!(
                "Found solution {} in {} steps after {elapsed:.2}s!",
                best_score.guess_str(),
                guesses.len()
            );
            Ok(guesses)
        } else {
            Err(format!(
                "Cannot find solution {}: best guess {}!",
                self.solution.guess_str(),
                guesses.last().unwrap().0.guess_str()
            ))
        }
    }

    pub fn calculate_initial_guess_performance_from(
        &mut self,
        initial_guess_str: &str,
        resume_file: Option<&str>,
    ) -> Result<(), String> {
        let initial_guess = WordClue::make_word_chars(initial_guess_str);

        if !self.solver.all_guesses.contains(&initial_guess) {
            return Err(format!(
                "Initial guess '{initial_guess_str}' is not in guess list!"
            ));
        }
        let file_name = resume_file
            .map(str::to_owned)
            .unwrap_or_else(|| format!("wordle_initial_guess_{initial_guess_str}_results.txt"));
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

            self.solution = GuessScore::new(0.0, false, *solution);
            let solution_str = self.solution.guess_str();
            if self.verbosity >= 1 {
                println!("Testing solution {solution_str}");
            }

            let result = self.run(initial_guess_str)?;
            total_steps += result.len();
            let mut line = format!("{solution_str}, {}", result.len());
            for (guess, count) in result {
                line.push_str(&format!(", {}({count})", guess.guess_str()));
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
    all_solutions: &BTreeSet<WordChars>,
) -> Result<(HashSet<WordChars>, usize), String> {
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
        let solution_str = fields[0];
        let solution = WordClue::make_word_chars(solution_str);
        if !solution_set.contains(&solution) {
            return Err(format!("Unknown solution '{solution_str}' in {file_name}"));
        }
        if !completed.insert(solution) {
            return Err(format!(
                "Duplicate solution '{solution_str}' in {file_name}"
            ));
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
