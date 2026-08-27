mod filter;
mod simulator;
mod solver;

use simulator::{BEST_INITIAL_GUESS, GameSim};
use solver::{DEFAULT_GUESSES_FILE, DEFAULT_SOLUTIONS_FILE, Solver, resolve_default_path};

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut solutions_file = resolve_default_path(DEFAULT_SOLUTIONS_FILE);
    let mut guesses_file = resolve_default_path(DEFAULT_GUESSES_FILE);
    let mut simulate_game = None;
    let mut hint_best = None;
    let mut force_guess = None;
    let mut calc_init = None;
    let mut resume_file = None;
    let mut quiet = false;
    let mut verbose = false;
    let mut hard_mode = false;
    let mut no_tests = false;
    let mut reverse = false;
    let mut filters = Vec::new();

    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        let mut value = |name: &str| -> Result<String, String> {
            index += 1;
            arguments
                .get(index)
                .cloned()
                .ok_or_else(|| format!("Missing value for {name}"))
        };
        match argument.as_str() {
            "-s" | "--solution-file" => solutions_file = value(argument)?.to_owned(),
            "-g" | "--guesses-file" => guesses_file = value(argument)?.to_owned(),
            "-m" | "--simulate-game" => simulate_game = Some(value(argument)?),
            "-b" | "--hint-best" => hint_best = Some(value(argument)?.to_uppercase()),
            "-f" | "--force-guess" => force_guess = Some(value(argument)?.to_uppercase()),
            "-c" | "--calc-init-guess" => calc_init = Some(value(argument)?.to_uppercase()),
            "--resume" => resume_file = Some(value(argument)?),
            "-q" | "--quiet" => quiet = true,
            "-v" | "--verbose" => verbose = true,
            "--hard-mode" => hard_mode = true,
            "--no-tests" => no_tests = true,
            "-r" | "--reverse" => reverse = true,
            "-h" | "--help" => return print_help(),
            argument if argument.starts_with('-') => {
                return Err(format!("Unknown option {argument}"));
            }
            filter => filters.push(filter.to_owned()),
        }
        index += 1;
    }

    if !no_tests {
        let mut test_solver = Solver::new(&solutions_file, &guesses_file, 0)?;
        test_solver.filter(&["BAC__".to_owned()])?;
    }

    let verbosity = if verbose {
        2
    } else if quiet {
        0
    } else {
        1
    };
    if let Some(solution) = calc_init {
        let mut sim = GameSim::new(&solution, &guesses_file, verbosity)?;
        return sim.calculate_initial_guess_performance_from(&solution, resume_file.as_deref());
    }
    if let Some(solution) = simulate_game {
        let mut sim = GameSim::new(&solution, &guesses_file, verbosity)?;
        return sim
            .run(force_guess.as_deref().unwrap_or(BEST_INITIAL_GUESS))
            .map(|_| ());
    }

    let mut solver = Solver::new(&solutions_file, &guesses_file, verbosity)?;
    if hint_best.is_some() && filters.is_empty() {
        filters.push(crate::filter::EMPTY_FILTER.to_owned());
    }
    if filters.is_empty() {
        return Ok(());
    }
    solver.filter(&filters)?;
    let num_sols = solver.len();
    println!("Filtered Solutions: {num_sols}");
    if num_sols > 0 && (solver.verbosity >= 2 || num_sols <= 10) {
        println!("{:?}", solver.filtered_sols);
    }

    let best_list = solver.find_best_guess(hard_mode, reverse)?;
    print!("Best guesses: ");
    for guess in best_list.into_iter().take(10) {
        print!("{}({:.2}), ", guess.0, guess.1);
    }
    println!();

    Ok(())
}

fn print_help() -> Result<(), String> {
    println!("Usage: rust_solver [options] [filters ...]");
    println!("  -s, --solution-file FILE    solution file");
    println!("  -g, --guesses-file FILE     guesses file");
    println!("  -m, --simulate-game WORD    simulate a game");
    println!("  -b, --hint-best WORD        seed the best-guess search");
    println!("  -f, --force-guess WORD      score a specific guess");
    println!("  -c, --calc-init-guess WORD  calculate initial-guess performance");
    println!("      --resume FILE           continue an existing performance output");
    println!("  -q, --quiet                 reduce output");
    println!("  -v, --verbose               increase output");
    println!("      --hard-mode             only use remaining solutions as guesses");
    println!("      --no-tests              skip startup checks");
    println!("  -r, --reverse               search guesses in reverse order");
    Ok(())
}
