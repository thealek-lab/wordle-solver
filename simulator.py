from solver import Solver

BEST_INITIAL_GUESS = "ROATE"

class GameSim:
   def __init__(self, solution: str, opts = None):

      self.verbose = False
      self.quiet = False
      if opts:
         self.verbose = opts.verbose
         self.quiet = opts.quiet
         
      if self.verbose:
         print(f"Creating GameSim with hidden solution {solution}")
         
      self.solution = solution
      
   def run(self, initial_guess: str = BEST_INITIAL_GUESS):

      sols = Solver(verbose=self.verbose, quiet=self.quiet)
      
      sols.filter([initial_guess])
      num_sols = len(sols)
      print(f"First guess {initial_guess} Solutions: {num_sols}")
