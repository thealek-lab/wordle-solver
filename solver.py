import sys, time

from filter import Filter
from guess_set import Guess, GuessSet

DEFAULT_SOLUTIONS_FILE = "./solutions.txt"
DEFAULT_GUESSES_FILE = "./all_guesses_2022_11K.txt"

class Solver:
   def __init__(self, solution_file_name: str = DEFAULT_SOLUTIONS_FILE, guesses_file_name: str = DEFAULT_GUESSES_FILE, verbosity: int = 1):
      if verbosity >= 2:
         print(f"Creating Solver with guesses {guesses_file_name}")

      self.verbosity = verbosity
      all_solutions = []
      with open(solution_file_name) as f:
         for l in f:
            line = l.strip().split()
            if len(line) == 2 and line[0] != "Word":
               sol = line[0].upper()
               if len(sol) != 5:
                  raise ValueError(f"Invalid solution length: {sol}")
               all_solutions.append(sol)

      if len(set(all_solutions)) != len(all_solutions):
         raise ValueError(f"Duplicate guess in solutions file: {solution_file_name}")

      self.all_solutions = sorted(all_solutions)
      self.filtered_sols = self.all_solutions.copy()

      if len(guesses_file_name) == 0:
         all_guesses = self.all_solutions.copy()
      else:
         all_guesses = []
         with open(guesses_file_name) as f:
            for l in f:
               line = l.strip().split()
               if len(line) == 1:
                  guess = line[0].upper()
                  if len(guess) != 5:
                     raise ValueError(f"Invalid guess length: {guess}")
                  all_guesses.append(guess)

         if len(set(all_guesses)) != len(all_guesses):
            raise ValueError(f"Duplicate guess in guesses file: {guesses_file_name}")

         all_guesses.extend(self.all_solutions.copy())

      for sol in self.all_solutions:
         if not sol in all_guesses:
            raise ValueError(f"Missing solution {sol} in guesses file: {guesses_file_name}")

      self.all_guesses = sorted(all_guesses)
      if self.verbosity >= 1:
         print(f"All Possible Solutions: {len(self.all_solutions)}")
         print(f"All Possible Guesses: {len(self.all_guesses)}")

   def __len__(self) -> int:
      return len(self.filtered_sols)

   def filter(self, filter_str_list: list[str]):
      filter_func_list = []
      for filt_str in filter_str_list:
         filter_func_list.append(Filter.make_filter_func(filt_str))

      for filt_func in filter_func_list:
         self.filtered_sols = list(filter(filt_func, self.filtered_sols))

      self.filtered_sols = sorted(self.filtered_sols)

   def try_guess(self, guess_str: str, lowest_remaining_cnt: int = sys.maxsize) -> GuessSet:

      partitions = {}
      hints = []
      for maybe_sol in self.filtered_sols:
         hint = Filter.make_hint(guess_str, maybe_sol)
         if not hint.is_match(guess_str):
            key = str(hint)
            partitions.setdefault(key, [])
            partitions[key].append(maybe_sol)
         hints.append((maybe_sol, hint))

      guess_set = GuessSet(guess_str, len(self.filtered_sols), is_in_tot_sol_set=True, is_in_remaining_sol_set=False)
      for maybe_sol, hint in hints:
         key = str(hint)
         remaining = [] if hint.is_match(guess_str) else partitions[key]
         guess_obj = Guess(maybe_sol, hint, remaining)
         if self.verbosity >= 2:
            print(f"Guess {guess_str} number remaining: {guess_obj.remaining_cnt} with solution {maybe_sol} hint {hint}")
         guess_set.add_guess(guess_obj)

         if guess_set.remaining_cnt > lowest_remaining_cnt:
            # No need to continue if the score is already worse than the best score
            break
      return guess_set

   def find_best_guess(self, hint_best: GuessSet = None, hard_mode: bool = False, reverse: bool = False) -> tuple[GuessSet, float]:
      start_time = time.perf_counter()

      solutions = self.filtered_sols
      num_sols =  len(solutions)
      if num_sols == 0:
         raise ValueError("No solutions left to guess from")

      # In hard mode, guesses must themselves satisfy all known clues.
      guess_list = solutions if hard_mode else self.all_guesses
      tot_guesses = len(guess_list)

      if reverse:
         guess_list = list(reversed(guess_list))

      best_guess = hint_best
      if hint_best is None:
         best_guess = self.try_guess(solutions[0])
         best_guess.is_in_remaining_sol_set = True
         if self.verbosity >=1:
            print(f"Initial guess {best_guess.to_str()}: expected solutions remaining {best_guess.remaining_avg:.2f} on average from {best_guess.remaining_cnt}/{num_sols}")
      best_score = best_guess.remaining_cnt

      last_time = start_time
      for i, guess_str in enumerate(guess_list):
         new_guess = self.try_guess(guess_str, best_score)
         new_guess.is_in_remaining_sol_set = guess_str in solutions
         new_guess.is_in_tot_sol_set = guess_str in self.all_solutions
         percent_complete = 100.0*(i+1)/tot_guesses
         loop_done_time = time.perf_counter()
         time_remaining = (loop_done_time - start_time) * (tot_guesses - i - 1) / (i + 1)

         if new_guess.is_better_than(best_guess, self.verbosity):
            best_score = new_guess.remaining_cnt
            best_guess = new_guess
            if self.verbosity >= 1:
               print(f"[{percent_complete:.2f}% after {loop_done_time - start_time:.2f}s done in {time_remaining:.2f}s] Best guess {best_guess.to_str()}: expected solutions remaining {best_guess.remaining_avg:.2f} on average from {best_guess.remaining_cnt}/{num_sols}")
            last_time = loop_done_time
         elif self.verbosity >= 1 and loop_done_time - last_time >= 5.0:
            print(f"[{percent_complete:.2f}% after {loop_done_time - start_time:.2f}s done in {time_remaining:.2f}s] Best guess is still {best_guess.to_str()} {best_guess.remaining_avg:.2f} (last guess was {guess_str} {new_guess.remaining_avg:.2f}+)")
            last_time = loop_done_time
            
      elapsed = time.perf_counter() - start_time
      
      if self.verbosity >=1:
         print(f"Time taken: {elapsed:.2f} seconds")
         if best_guess.guess_str not in self.all_solutions:
            print(f"WARNING: Best guess {best_guess.to_str()} is not in the solutions file!")
         print(f"Best guess {best_guess.to_str()}: expected solutions remaining {best_guess.remaining_avg:.2f} on average from {best_guess.remaining_cnt}/{num_sols}")

      return best_guess


   def unit_tests(self):
      # Test the filter function
      test_cases = [
         ("BAC__", ["BACON"]),
         ("_ALE", ['VALET', 'PALER', 'BALER', 'BALED', 'PALED']),
         ("_ALEb", ['BALER', 'BALED']),
         ("alek", []),
         ("_alek", ['ALIKE', 'ANKLE', 'FLAKE', 'LEAKY', 'SLAKE']),
         ("_kela", ['ALIKE', 'FLAKE', 'LEAKY', 'SLAKE', 'LATKE']),
         ("Alek", ['ANKLE']),
      ]
      for filt, expected in test_cases:
         self.filtered_sols = self.all_solutions.copy()
         self.filter([filt])
         assert set(self.filtered_sols) == set(expected), f"Filter {filt} failed. Expected {expected}, got {self.filtered_sols}"
