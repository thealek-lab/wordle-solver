#! /usr/bin/env python3

import os
import sys
from optparse import OptionParser

from filter import EMPTY_FILTER
from solver import Solver, DEFAULT_GUESSES_FILE, DEFAULT_SOLUTIONS_FILE
from simulator import GameSim

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

if __name__ == "__main__":
   os.chdir(SCRIPT_DIR)

   parser = OptionParser(usage="%prog [options] [filters ...]")
   parser.add_option("-s", "--solution-file", action="store", dest="solutions_file",
                     default="./solutions.txt", help="solution file (default: ./solutions.txt)")
   parser.add_option("-g", "--guesses-file", action="store", dest="guesses_file",
                     default=DEFAULT_GUESSES_FILE, help=f"guesses file (default: {DEFAULT_GUESSES_FILE})")
   parser.add_option("-m", "--simulate-game", action="store", dest="simulate_game",
                     help=f"hidden solution for this simulation")
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
   opts, args = parser.parse_args()

   if not opts.no_tests:
      sols = Solver(opts.solutions_file, verbosity=0)
      sols.unit_tests()
      
   if opts.simulate_game:
      sim = GameSim(opts.simulate_game, opts)
      sim.run()
      sys.exit(0)
      
   verbosity = 2 if opts.verbose else 0 if opts.quiet else 1

   sols = Solver(opts.solutions_file, opts.guesses_file, verbosity=verbosity)

   if opts.hint_best and len(args) == 0:
      # Force search through the entire set of solutions if a hint best guess is provided but no filters are given
      args.append(EMPTY_FILTER)

   if len(args) > 0:
      # Apply the solution filters
      sols.filter(args)
      num_sols = len(sols)
      print(f"Filtered Solutions: {num_sols}")
      if num_sols > 0 and (sols.verbosity >= 2 or num_sols <= 10):
         print(f"{sols.filtered_sols}")

      if opts.force_guess:
         if opts.force_guess not in sols.filtered_sols:
            print(f"WARNING: Forced guess {opts.force_guess} is not in the filtered solutions!")

         best_guess = sols.try_guess(opts.force_guess)
         best_guess.is_in_remaining_sol_set = best_guess.guess_str in sols.filtered_sols
         best_guess.is_in_tot_sol_set = best_guess.guess_str in sols.all_solutions
         print(f"Forced guess {best_guess.to_str()}: expected solutions remaining {best_guess.remaining_avg:.2f} on average from {best_guess.remaining_cnt}/{num_sols}")
      else:
         hint_obj = None
         if opts.hint_best:
            if opts.hint_best not in sols.filtered_sols:
               print(f"WARNING: Hint best guess {opts.hint_best} is not in the filtered solutions!")

            hint_obj = sols.try_guess(opts.hint_best)
            hint_obj.is_in_remaining_sol_set = hint_obj.guess_str in sols.filtered_sols
            hint_obj.is_in_tot_sol_set = hint_obj.guess_str in sols.all_solutions
            print(f"Hinted best guess {hint_obj.to_str()}: expected solutions remaining {hint_obj.remaining_avg:.2f} on average from {hint_obj.remaining_cnt}/{num_sols}")

         best_guess, elapsed = sols.find_best_guess(hard_mode=opts.hard_mode, hint_best=hint_obj, reverse=opts.reverse)
         print(f"Time taken: {elapsed:.2f} seconds")
         if best_guess.guess_str not in sols.all_solutions:
            print(f"WARNING: Best guess {best_guess.to_str()} is not in the solutions file!")
         print(f"Best guess {best_guess.to_str()}: expected solutions remaining {best_guess.remaining_avg:.2f} on average from {best_guess.remaining_cnt}/{num_sols}")

      if sols.verbose >= 2 or len(sols) < 30:
         for guess_obj in best_guess.guesses:
            if guess_obj.hint.is_match(best_guess.guess_str):
               print(f"   Solution: {guess_obj.maybe_sol} Hint: {guess_obj.hint} SUCCESS")
            else:
               print(f"   Solution: {guess_obj.maybe_sol} Hint: {guess_obj.hint} {guess_obj.remaining_cnt} remaining solutions {guess_obj.remaining_sol_set}")
