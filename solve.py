#! /usr/bin/env python3

import os
import sys
import time
from optparse import OptionParser

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
EXCLUSION_CHAR = "^"
UNKNOWN_CHAR = "_"
EMPTY_FILTER = UNKNOWN_CHAR * 5

class Guess:
   def __init__(self, maybe_sol: str, hint: str, exclude: str, remaining_sol_list: list):
      self.maybe_sol = maybe_sol
      self.hint = hint
      self.exclude = exclude
      self.remaining_sol_list = remaining_sol_list
      self.remaining_cnt = len(remaining_sol_list)

class GuessSet:
   def __init__(self, guess_str: str, num_sols: int):
      self.guess_str = guess_str
      self.guesses = []
      self.num_sols = num_sols
      self.remaining_cnt = sys.maxsize
      self.remaining_avg = float('inf')

   def add_guess(self, guess: Guess):
      self.guesses.append(guess)
      if self.remaining_cnt == sys.maxsize:
         self.remaining_cnt = 0
      self.remaining_cnt += len(guess.remaining_sol_list)
      self.remaining_avg = self.remaining_cnt / self.num_sols

class Solutions:
   def __init__(self, solution_file_name: str, verbose: bool = False, quiet: bool = False):
      self.verbose = 2 if verbose else 0 if quiet else 1
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

   def __len__(self) -> int:
      return len(self.filtered_sols)

   def make_exclude_filter(self, exclude_str: str):
      assert exclude_str.isalpha(), f"Excluded characters {exclude_str} are not alphabetic"

      def filter_func(s: str) -> bool:
         for c in exclude_str.upper():
            if c in s:
               if self.verbose >= 3:
                  print(f"Discarding {s} because it contains excluded character {c}")
               return False
         return True

      return filter_func

   def make_filter_func(self, filter_str: str):
      if len(filter_str) == 0 or filter_str == EMPTY_FILTER:
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
      hint = list(EMPTY_FILTER)
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

   def try_guess(self, guess_str: str, lowest_remaining_cnt: int = sys.maxsize) -> GuessSet:

      guess_set = GuessSet(guess_str, len(self.filtered_sols))
      partitions = {}
      hints = []
      for maybe_sol in self.filtered_sols:
         hint, exclude = self.make_hint(guess_str, maybe_sol)
         if hint != guess_str:
            key = (hint, exclude)
            partitions.setdefault(key, []).append(maybe_sol)
         hints.append((maybe_sol, hint, exclude))

      for maybe_sol, hint, exclude in hints:
         remaining = [] if hint == guess_str else partitions[(hint, exclude)]
         guess_obj = Guess(maybe_sol, hint, exclude, remaining)
         if self.verbose >= 2:
            print(f"Guess {guess_str} number remaining: {guess_obj.remaining_cnt} with solution {maybe_sol} hint {hint}")
         guess_set.add_guess(guess_obj)

         if guess_set.remaining_cnt > lowest_remaining_cnt:
            # No need to continue if the score is already worse than the best score
            break
      return guess_set

   def find_best_guess(self, solutions: list[str], hint_best: GuessSet = None, hard_mode: bool = False, reverse: bool = False) -> tuple[GuessSet, float]:
      start_time = time.perf_counter()

      if len(solutions) == 0:
         raise ValueError("No solutions left to guess from")

      # In hard mode, guesses must themselves satisfy all known clues.
      guess_list = solutions if hard_mode else self.all_solutions
      tot_guesses = len(guess_list)
      
      if reverse:
         guess_list = list(reversed(guess_list))

      # Count hint partitions directly instead of cloning and filtering the
      # solution set once for every possible hint.
      best_guess = hint_best
      best_score = hint_best.remaining_cnt if hint_best is not None else sys.maxsize
      filtered_sols = self.filtered_sols

      last_time = start_time
      for i, guess_str in enumerate(guess_list):
         partition_counts = {}
         for maybe_sol in filtered_sols:
            hint, exclude = self.make_hint(guess_str, maybe_sol)
            if hint == guess_str:
               continue
            key = (hint, exclude)
            partition_counts[key] = partition_counts.get(key, 0) + 1

         score = sum(count * count for count in partition_counts.values())
         guess_set = GuessSet(guess_str, len(filtered_sols))
         guess_set.remaining_cnt = score
         guess_set.remaining_avg = score / len(filtered_sols)
         percent_complete = 100.0*(i+1)/tot_guesses
         loop_done_time = time.perf_counter()
         time_remaining = (loop_done_time - start_time) * (tot_guesses - i - 1) / (i + 1)
         if guess_set.remaining_cnt < best_score:
            best_score = guess_set.remaining_cnt
            best_guess = guess_set
            if self.verbose >= 1:
               print(f"[{percent_complete:.2f}% after {loop_done_time - start_time:.2f}s done in {time_remaining:.2f}s] Best guess {best_guess.guess_str}: expected solutions remaining {best_guess.remaining_avg:.2f} on average from {best_guess.remaining_cnt}/{tot_guesses}")
            last_time = loop_done_time
         elif self.verbose >= 2:
            print(f"Guess {guess_str} score of {guess_set.remaining_avg:.2f} with {tot_guesses} solutions")
         elif self.verbose >= 1 and loop_done_time - last_time >= 5.0:
            print(f"[{percent_complete:.2f}% after {loop_done_time - start_time:.2f}s done in {time_remaining:.2f}s] Best guess is still {best_guess.guess_str} {best_guess.remaining_avg:.2f} (last guess was {guess_str} {guess_set.remaining_avg:.2f})")
            last_time = loop_done_time
      return (best_guess, loop_done_time - start_time)

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

   parser = OptionParser(usage="%prog [options] [filters ...]")
   parser.add_option("-s", "--solution-file", action="store", dest="solution_file",
                     default="./solutions.txt", help="solution file (default: ./solutions.txt)")
   parser.add_option("-b", "--hint-best", action="store", dest="hint_best",
                     help="hint best guess (e.g. 'SLATE' or 'RAISE')")
   parser.add_option("-f", "--force-guess", action="store", dest="force_guess",
                     help="force guess (e.g. 'SLATE' or 'RAISE')")
   parser.add_option("-q", "--quiet", action="store_true", dest="quiet",
                     default=False, help="disable most output")
   parser.add_option("-v", "--verbose", action="store_true", dest="verbose",
                     default=False, help="enable verbose output")
   parser.add_option("--hard-mode", action="store_true", dest="hard_mode",
                     default=False, help="only use guesses satisfying known clues")
   parser.add_option("--no-tests", action="store_true", dest="no_tests",
                     default=False, help="skip unit tests")
   parser.add_option("-r", "--reverse", action="store_true", dest="reverse",
                     default=False, help="search in reverse order")
   options, args = parser.parse_args()

   if not options.no_tests:
      sols = Solutions(options.solution_file, verbose=False, quiet=False)
      sols.unit_tests()

   sols = Solutions(options.solution_file, verbose=options.verbose, quiet=options.quiet)
   print(f"All Possible Solutions: {len(sols.all_solutions)}")

   if options.hint_best and len(args) == 0:
      # Force search through the entire set of solutions if a hint best guess is provided but no filters are given
      args.append(EMPTY_FILTER)

   if len(args) > 0:
      # Apply the solution filters
      sols.filter(args)
      num_sols = len(sols)
      print(f"Filtered Solutions: {num_sols}")
      if num_sols > 0 and (sols.verbose >= 2 or num_sols <= 10):
         print(f"{sols.filtered_sols}")

      if options.force_guess:
         if options.force_guess not in sols.filtered_sols:
            print(f"WARNING: Forced guess {options.force_guess} is not in the filtered solutions!")

         hint_obj = sols.try_guess(options.force_guess)
         best_guess = GuessSet(options.force_guess, num_sols)
         print(f"Forced guess {best_guess.guess_str}:expected solutions remaining {hint_obj.remaining_avg:.2f} on average from {hint_obj.remaining_cnt}/{num_sols}")
      else:
         hint_obj = None
         if options.hint_best:
            if options.hint_best not in sols.filtered_sols:
               print(f"WARNING: Hint best guess {options.hint_best} is not in the filtered solutions!")

            hint_obj = sols.try_guess(options.hint_best)
            print(f"Hinted best guess {options.hint_best}: expected solutions remaining {hint_obj.remaining_avg:.2f} on average from {hint_obj.remaining_cnt}/{num_sols}")

         best_guess, elapsed = sols.find_best_guess(sols.filtered_sols, hard_mode=options.hard_mode, hint_best=hint_obj, reverse=options.reverse)
         print(f"Time taken: {elapsed:.2f} seconds")
         print(f"Best guess {best_guess.guess_str}: expected solutions remaining {best_guess.remaining_avg:.2f} on average from {best_guess.remaining_cnt}/{num_sols}")

      if sols.verbose >= 2 or len(sols) < 30:
         for guess_obj in sols.try_guess(best_guess.guess_str).guesses:
            if guess_obj.hint == best_guess.guess_str:
               print(f"   Solution: {guess_obj.maybe_sol} Hint: {guess_obj.hint} SUCCESS")
               continue
            print(f"   Solution: {guess_obj.maybe_sol} Hint: {guess_obj.hint}{guess_obj.exclude} {guess_obj.remaining_cnt} remaining solutions {guess_obj.remaining_sol_list}")
