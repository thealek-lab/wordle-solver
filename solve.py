#! /usr/bin/env python3

import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

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
                
      def filter_func(s: str) -> bool:
         for c in exclude_str.upper():
            if c in s:
               if self.verbose:
                  print(f"Discarding {s} because it contains excluded character {c}")
               return False
         return True
            
      return filter_func
   
   def make_filter_func(self, filter_str: str):
      if filter_str[0] == "~":
         return self.make_exclude_filter(filter_str[1:])
      filter_str = list(filter_str)
      # Split the filter string into upper and all chars
      all_chars = []
      for i, c in enumerate(filter_str):
         if c != "_":
            if not c.isalpha():
               raise ValueError(f"Invalid character in filter: {c}")
            if c.islower():
               filter_str[i]="_"
            all_chars.append(c.upper())
            
      def filter_func(s: str) -> bool:
         for i, c in enumerate(filter_str):
            if c != "_":
               if c != s[i]:
                  return False
               if c in all_chars:
                  all_chars.remove(c)

         for c in all_chars:
            c_cnt = Solutions.count_chars(s, c)
            s_cnt = Solutions.count_chars(all_chars, c)
            if c_cnt != s_cnt:
               if self.verbose:
                  print(f"Discarding {s} because it does not have the right number {s_cnt} of {c}'s")
               return False
         return True
      return filter_func
   
   def filter(self, filter_list: list[str]):
      for filt in filter_list:
         if len(filt) == 0:
            # Empty filter, skip it
            continue
         if filt[0] != "~" and len(filt) > 5:
            raise ValueError(f"Invalid filter length: {filt}")
         filter_func = self.make_filter_func(filt)
         self.filtered_sols = list(filter(filter_func, self.filtered_sols))
      self.filtered_sols = sorted(self.filtered_sols)

   def make_hint(self, guess: str, solution: str):
      sol = list(solution)
      # Make a hint based on the guess and the solution
      hint = ["_"] * 5
      exclude = []
      for i, c in enumerate(guess):
         if c == sol[i]:
            hint[i] = c.upper()
            sol[i] = "_"
         else:
            try:
               pos = sol.index(c)
            except ValueError:
               if not c in solution:
                  exclude.append(c)
               continue
            hint[i] = c.lower()
            sol[pos] = "_"
      if len(exclude) > 0:
         exclude = ["~"] + sorted(set(exclude))
      return ("".join(hint), "".join(exclude))
      
   def find_best_guess(self, solutions: list[str]) -> str:
      tot_sols = len(solutions)
      # Find the best guess by calculating the probabilities of guessing in each step
      best_guess = ""
      best_score = float('inf')
      for guess in self.all_solutions:
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
         ("alek", ['ALIKE', 'ANKLE', 'BLEAK', 'FLAKE', 'LEAKY', 'SLAKE', 'LATKE']),
         ("_alek", ['ALIKE', 'ANKLE', 'BLEAK', 'FLAKE', 'LEAKY', 'SLAKE', 'LATKE']),
         ("_kela", ['ALIKE', 'ANKLE', 'BLEAK', 'FLAKE', 'LEAKY', 'SLAKE', 'LATKE']),
         ("Alek", ['ALIKE', 'ANKLE']),
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
         
      best_guess = sols.find_best_guess(sols.filtered_sols)
      print(f"Best Guess: {best_guess}")
      
      if len(sols) < 30:
         for s in sols.filtered_sols:
            hint, exclude = sols.make_hint(best_guess, s)
            if hint==best_guess:
               print(f"   Solution: {s} Hint: {hint} SUCCESS")
               continue
            clone = sols.clone()
            clone.filter([hint, exclude])
            print(f"   Solution: {s} Hint: {hint} Exclude: '{exclude}' {len(clone.filtered_sols)} remaining solutions {clone.filtered_sols}")
