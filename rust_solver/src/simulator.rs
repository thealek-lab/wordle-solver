use std::collections::HashSet;
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
    pub fn new(solution: &str, guesses_file: &str, verbosity: u32) -> Result<Self, String> {
        if verbosity >= 2 {
            println!("Creating GameSim with hidden solution {solution}");
        }

        let solution = WordClue::make_word_chars(solution);
        let solution = GuessScore::new(solution, 0.0, false);

        let solver = Solver::new(
            &resolve_default_path(DEFAULT_SOLUTIONS_FILE),
            guesses_file,
            verbosity,
        )?;
        if !solver.all_solutions.contains(&solution.word_chars) {
            return Err(format!(
                "Invalid solution {} is not in the official list!",
                solution.guess_str()
            ));
        }

        Ok(Self {
            solution,
            verbosity,
            solver,
        })
    }

    pub fn run(&mut self, initial_guess_str: &str) -> Result<Vec<(GuessScore, usize)>, String> {
        let start_time = Instant::now();

        let sol_chars = self.solution.word_chars;
        let solution_str = self.solution.guess_str();

        let initial_guess = WordClue::make_word_chars(initial_guess_str);

        self.solver.filtered_sols = self.solver.all_solutions.clone();

        self.solver
            .filter_by_guess_n_solution(initial_guess, sol_chars)?;
        let mut guesses = vec![(
            GuessScore::new(initial_guess, 0.0, false),
            self.solver.len(),
        )];
        if self.verbosity >= 1 {
            println!(
                "After first guess {initial_guess_str} Solutions: {}",
                self.solver.len()
            );
        }
        while !self.solver.is_empty() {
            let ranked_guesses = self.solver.find_best_guess(false, false, 1)?;
            let score = &ranked_guesses[0];
            self.solver
                .filter_by_guess_n_solution(score.word_chars, sol_chars)?;
            guesses.push((score.clone(), self.solver.len()));
            if self.verbosity >= 1 {
                println!(
                    "After guess {} {} Solutions: {}",
                    guesses.len(),
                    score.guess_str(),
                    self.solver.len()
                );
            }
            if self.verbosity >= 2 && !self.solver.is_empty() && self.solver.len() <= 10 {
                println!(
                    "   {}",
                    self.solver
                        .filtered_sols
                        .iter()
                        .map(|a| a.iter().collect::<String>())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if self.solver.len() == 1 {
                let solution = self.solver.filtered_sols[0];
                if guesses.last().unwrap().0.word_chars != solution {
                    guesses.push((GuessScore::new(solution, 100.0, true), 0));
                }
                break;
            }
            if guesses.len() >= 7 {
                break;
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let last_guess = guesses.last();
        if let Some((best_guess, _)) = last_guess
            && best_guess.word_chars == sol_chars
        {
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

            let solution = GuessScore::new(*solution, 0.0, false);
            let sol_str = solution.guess_str();

            if self.verbosity >= 1 {
                println!("Testing solution {sol_str}");
            }
            self.solution = solution;

            let result = self.run(initial_guess_str)?;
            total_steps += result.len();
            let mut line = format!("{sol_str}, {}", result.len());
            for (guess, count) in result {
                let count_str = if count == 0 { "*" } else { &count.to_string() };
                line.push_str(&format!(", {}({count_str})", guess.guess_str()));
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
    all_solutions: &[WordChars],
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
