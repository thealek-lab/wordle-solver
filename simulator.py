import time

from filter import Filter
from solver import Solver

BEST_INITIAL_GUESS = "ROATE"

class GameSim:
   def __init__(self, solution: str, verbosity: int = 1) -> bool:
      if verbosity >= 2:
         print(f"Creating GameSim with hidden solution {solution}")
         
      self.verbosity = verbosity
      self.solution = solution.upper()
      
   def run(self, initial_guess: str = BEST_INITIAL_GUESS):
      start_time = time.perf_counter()

      sols = Solver(verbosity=self.verbosity)
      
      if self.solution not in sols.all_solutions:
         raise ValueError(f"Invalid solution {self.solution} is not in the official list!")

      hint = Filter.make_hint(initial_guess, self.solution)
      
      hint_list = [str(hint)]
      sols.filter(hint_list)
      num_sols = len(sols)
      print(f"After first guess {initial_guess} Solutions: {num_sols}")

      best_guess = sols.find_best_guess()
      
      steps = 2
      while len(sols) > 1:
         hint = Filter.make_hint(best_guess.guess_str, self.solution)
         hint_list += str(hint),
         sols.filter(hint_list)
         num_sols = len(sols)
               
         best_guess = sols.find_best_guess()
         if len(sols) <= 1 or steps >= 5:
            break
         
         print(f"After guess {steps} {best_guess.guess_str} Solutions: {num_sols}")
         if num_sols > 0 and (self.verbosity >= 2 or num_sols <= 10):
            print(f"   {sols.filtered_sols}")
         steps += 1
      
      elapsed = time.perf_counter() - start_time

      found = best_guess.guess_str == self.solution
      if found:
         print(f"Found solution {best_guess.guess_str} in {steps} steps after {elapsed:.2f}s!")
      else:
         raise ValueError(f"Cannot find solution {self.solution}!")

      return found

