import time

from filter import Filter
from solver import Solver

BEST_INITIAL_GUESS = "ROATE"

class GameSim:
   def __init__(self, solution: str, verbosity: int = 1):
      if verbosity >= 2:
         print(f"Creating GameSim with hidden solution {solution}")
         
      self.verbosity = verbosity
      self.solution = solution.upper()
      
   def run(self, initial_guess: str = BEST_INITIAL_GUESS) -> list:
      start_time = time.perf_counter()

      sols = Solver(verbosity=self.verbosity)
      num_sols = len(sols)

      if self.solution not in sols.all_solutions:
         raise ValueError(f"Invalid solution {self.solution} is not in the official list!")

      hint = Filter.make_hint(initial_guess, self.solution)
      
      hint_list = [str(hint)]
      sols.filter(hint_list)
      num_sols = len(sols)
      guess_list = [(initial_guess, num_sols)]

      print(f"After first guess {initial_guess} Solutions: {num_sols}")

      while num_sols >= 1:
         best_guess = sols.find_best_guess()

         hint = Filter.make_hint(best_guess.guess_str, self.solution)
         hint_list += str(hint),
         sols.filter(hint_list)
         num_sols = len(sols)
         guess_list += (best_guess.guess_str, num_sols),

         print(f"After guess {len(guess_list)} {best_guess.guess_str} Solutions: {num_sols}")
         if num_sols > 0 and (self.verbosity >= 2 or num_sols <= 10):
            print(f"   {sols.filtered_sols}")

         if num_sols == 1:
            single_sol = sols.filtered_sols[0]
            if best_guess.guess_str != single_sol:
               guess_list += (single_sol, "*"),
            break

         if len(guess_list) >= 7:
            break
      
      elapsed = time.perf_counter() - start_time

      found = guess_list[-1][0] == self.solution
      if found:
         print(f"Found solution {guess_list[-1]} in {len(guess_list)} steps after {elapsed:.2f}s!")
      else:
         raise ValueError(f"Cannot find solution {self.solution}: best guess {guess_list[-1]}!")

      return guess_list

   @classmethod
   # Run simulations for all possible solutions and output a CSV file with the results
   def calculate_initial_guess_performance(cls, initial_guess: str = BEST_INITIAL_GUESS, verbosity: int = 1):
      
      initial_guess = initial_guess.upper()
      assert initial_guess.isalpha(), f"Initial guess '{initial_guess}' is not alphabetic"

      sols = Solver(verbosity=verbosity)
      assert initial_guess in sols.all_guesses, f"Initial guess '{initial_guess}' is not in guess list!"

      with open(f"wordle_initial_guess_{initial_guess}_results.txt", "w") as f:

         tot_guesses = len(sols.all_solutions)
         start_time = time.perf_counter()
         tot_steps = 0
         for i, sol in enumerate(sols.all_solutions):
            print(f"Testing solution {sol}")
            sim = GameSim(sol, verbosity=verbosity)
            guess_list = sim.run(initial_guess)
            steps = len(guess_list)
            csv_line = f"{sol}, {steps}"
            for guess, num_sols in guess_list:
               csv_line += f", {guess}({num_sols})"
               
            print(csv_line, file=f)
            f.flush()
            
            tot_steps += steps
            avg_steps = tot_steps / (i + 1)
            percent_complete = 100.0*(i+1)/tot_guesses
            loop_done_time = time.perf_counter()
            time_remaining = (loop_done_time - start_time) * (tot_guesses - i - 1) / (i + 1)
            
            print(f"[{percent_complete:.2f}% after {loop_done_time - start_time:.2f}s done in {time_remaining:.2f}s] avg={avg_steps:.2f}={tot_steps}/{i+1} ", end="")

