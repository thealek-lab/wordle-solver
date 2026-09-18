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

    pub fn calculate_initial_guess_performance_from(
        &mut self,
        initial_guess_str: &str,
    ) -> Result<(), String> {
        let initial_guess = WordClue::make_word_chars(initial_guess_str);

        if !self.solver.all_guesses.contains(&initial_guess) {
            return Err(format!(
                "Initial guess '{initial_guess_str}' is not in guess list!"
            ));
        }

        let file_name = format!("wordle_initial_guess_{initial_guess_str}_results.txt");

        let res = read_completed_results(&file_name, &self.solver.all_solutions);
        let mut output = if res.is_ok() {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_name)
        } else {
            File::create(&file_name)
        }
        .map_err(|error| format!("Could not open {file_name}: {error}"))?;

        let (last_sol, mut total_steps, mut completed_count) = res.unwrap();

        let total = self.solver.all_solutions.len();
        let start_time = Instant::now();
        for solution in &self.solver.all_solutions.clone() {
            if *solution <= last_sol {
                continue;
            }

            let solution = GuessScore::new(*solution, 0.0, false);
            let sol_str = solution.guess_str();

            if self.verbosity >= 1 {
                println!("Testing solution {sol_str}");
            }
            self.solution = solution;

            let result =
                self.solver
                    .simulate(initial_guess_str, &self.solution.guess_str(), false)?;
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
) -> Result<(WordChars, usize, usize), String> {
    let res = std::fs::read_to_string(file_name);
    if let Err(e) = res {
        eprintln!("WARNING: Could not read resume file {file_name}: {e}");
        return Ok((WordClue::make_word_chars("AAAAA"), 0, 0));
    }

    let mut total_steps = 0;
    let mut completed_count = 0;
    let mut last_line = vec![];
    let contents = res.unwrap();

    for (line_number, line) in contents.lines().enumerate() {
        let fields: Vec<_> = line.split(',').map(str::trim).collect();
        if fields.len() > 2 {
            total_steps += fields[1].parse::<usize>().map_err(|error| {
                format!(
                    "Invalid step count in resume row {}: {error}",
                    line_number + 1
                )
            })?;
            completed_count += 1;
            last_line = fields
        }
    }

    let solution = if last_line.is_empty() {
        all_solutions[0]
    } else {
        let last_line_str = last_line[0];
        let solution = WordClue::make_word_chars(last_line_str);

        if !all_solutions.binary_search(&solution).is_ok() {
            return Err(format!("Unknown solution '{last_line_str}' in {file_name}"));
        }

        solution
    };

    Ok((solution, total_steps, completed_count))
}
