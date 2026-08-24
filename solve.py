#! /usr/bin/env python3

import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
EXCLUSION_CHAR = "^"
UNKNOWN_CHAR = "_"

class Solutions:
   def __init__(self, solution_file_name: str, verbose: bool = False):
      self.verbose = verbose
      self.all_solutions = []
      with open(solution_file_name) as f:
         for l in f:
            line = l.strip().split()
            if len(line) == 2 and line[0] != "Word":
               sol = line[0].upper()
               if len(sol) != 5:
                  raise ValueError(f"Invalid solution length: {sol}")
               self.all_solutions.append(sol)

      self.all_solutions = sorted(self.all_solutions)
      self.filtered_sols = self.all_solutions.copy()

   def clone(self):
      new_sols = Solutions.__new__(Solutions)
      new_sols.verbose = self.verbose
      new_sols.all_solutions = self.all_solutions.copy()
      new_sols.filtered_sols = self.filtered_sols.copy()
      return new_sols

   def __len__(self) -> int:
      return len(self.filtered_sols)

   def count_chars(string: list, c: str) -> int:
      return string.count(c.upper())

   def make_exclude_filter(self, exclude_str: str):
      assert exclude_str.isalpha(), f"Excluded characters {exclude_str} are not alphabetic"

      def filter_func(s: str) -> bool:
         for c in exclude_str.upper():
            if c in s:
               if self.verbose:
                  print(f"Discarding {s} because it contains excluded character {c}")
               return False
         return True

      return filter_func

   def make_filter_func(self, filter_str: str):
      if len(filter_str) == 0 or filter_str == UNKNOWN_CHAR * 5:
         # Empty filter string, accept all solutions
         return lambda s: True

      if filter_str[0] == EXCLUSION_CHAR:
         return self.make_exclude_filter(filter_str[1:])

      assert len(filter_str) <= 5, f"Filter string '{filter_str}' is more than 5 characters long"

      stripped_filter = filter_str.replace(UNKNOWN_CHAR, "")
      assert stripped_filter.isalpha(), f"Filter string '{filter_str}' is not alphabetic"

      filter_str = list(filter_str)

      def filter_func(maybe_sol: str) -> bool:
         assert maybe_sol.isalpha(), f"Possible solution {maybe_sol} is not alphabetic"
         assert maybe_sol.upper() == maybe_sol, f"Possible solution {maybe_sol} is not uppercase"

         # Clone the possible solution so that we can edit it
         maybe_sol = list(maybe_sol)

         # Loop through known position characters (uppercase)
         for i, c in enumerate(filter_str):
            if c != UNKNOWN_CHAR and c.isupper():
               if c != maybe_sol[i]:
                  # A known position character is missing in the candidate
                  return False
               maybe_sol[i] = UNKNOWN_CHAR

         # Loop through unknown position characters (lowercase)
         for i, c in enumerate(filter_str):
            if c != UNKNOWN_CHAR and c.islower():
               up_c = c.upper()
               if up_c == maybe_sol[i]:
                  # A character is present in an excluded position within the candidate
                  return False
               try:
                  pos = maybe_sol.index(up_c)
                  maybe_sol[pos] = UNKNOWN_CHAR
               except ValueError:
                  # A character is missing from the candidate
                  return False

         return True
      return filter_func

   def filter(self, filter_list: list[str]):
      for filt in filter_list:
         filter_func = self.make_filter_func(filt)
         self.filtered_sols = list(filter(filter_func, self.filtered_sols))
      self.filtered_sols = sorted(self.filtered_sols)

   def make_hint(self, guess: str, solution: str):
      sol = list(solution)
      # Make a hint based on the guess and the solution
      hint = [UNKNOWN_CHAR] * 5
      exclude = []
      for i, c in enumerate(guess):
         if c == sol[i]:
            hint[i] = c.upper()
            sol[i] = UNKNOWN_CHAR
         else:
            try:
               pos = sol.index(c)
            except ValueError:
               if not c in solution:
                  exclude.append(c)
               continue
            hint[i] = c.lower()
            sol[pos] = UNKNOWN_CHAR
      if len(exclude) > 0:
         exclude = [EXCLUSION_CHAR] + sorted(set(exclude))
      return ("".join(hint), "".join(exclude))

   def try_guess(self, guess: str):
      results = []
      for maybe_sol in self.filtered_sols:
         hint, exclude = self.make_hint(guess, maybe_sol)
         if hint==best_guess:
            results.append((maybe_sol, hint, exclude, self.filtered_sols))
            continue
         clone = sols.clone()
         clone.filter([hint, exclude])
         results.append((maybe_sol, hint, exclude, clone.filtered_sols))

      return results

   def find_best_guess(self, solutions: list[str], hard_mode: bool = False) -> str:
      tot_sols = len(solutions)
      # Find the best guess by calculating the probabilities of guessing in each step
      best_guess = ""
      best_score = float('inf')
      # In hard mode, guesses must themselves satisfy all known clues.
      guesses = solutions if hard_mode else self.all_solutions
      for guess in guesses:
         score=0
         for s in solutions:
            hint, exclude = self.make_hint(guess, s)
            if guess == hint:
               continue
            clone = self.clone()
            clone.filter([hint, exclude])
            remaining_sols = len(clone.filtered_sols)
            if remaining_sols == 0:
               # Bad guess with no solutions left, skip it
               continue
            score += len(clone.filtered_sols)
            #print(f"Guess {guess} score of {score} with solution {s} hint {hint}")
         if score < best_score:
            best_score = score
            best_guess = guess
            print(f"Best guess {guess} with expected solutions remaining {score/len(solutions):.2f} on average")
         elif self.verbose:
            print(f"Guess {guess} score of {score} with {len(solutions)} solutions")
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

if __name__ == "__main__":
   os.chdir(SCRIPT_DIR)

   verbose = "--verbose" in sys.argv[1:]
   if verbose:
      sys.argv.remove("--verbose")

   hard_mode = "--hard-mode" in sys.argv[1:]
   if hard_mode:
      sys.argv.remove("--hard-mode")

   if "--no-tests" in sys.argv[1:]:
      sys.argv.remove("--no-tests")
   else:
      sols = Solutions("./solutions.txt", verbose=False)
      sols.unit_tests()

   if len(sys.argv) < 2:
      print("Usage: solve.py <solution_file>")
      sys.exit(1)
   solution_file_name = sys.argv[1]

   sols = Solutions(solution_file_name, verbose=verbose)
   print(f"All Possible Solutions: {len(sols.all_solutions)}")

   if len(sys.argv) > 2:
      # Apply the solution filters
      sols.filter(sys.argv[2:])
      num_sols = len(sols)
      print(f"Filtered Solutions: {num_sols}")
      if num_sols>0 and num_sols < 10:
         print(f"{sols.filtered_sols}")

      best_guess = sols.find_best_guess(sols.filtered_sols, hard_mode=hard_mode)
      print(f"Best Guess: {best_guess}")

      if len(sols) < 31:
         for s, hint, exclude, remaining in sols.try_guess(best_guess):
            if hint==best_guess:
               print(f"   Solution: {s} Hint: {hint} SUCCESS")
               continue
            print(f"   Solution: {s} Hint: {hint} Exclude: '{exclude}' {len(remaining)} remaining solutions {remaining}")
