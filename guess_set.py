import sys

from filter import Filter

class Guess:
   def __init__(self, maybe_sol: str, hint: Filter, remaining_sol_set: list):
      self.maybe_sol = maybe_sol
      self.is_in_remaining_sol_set = maybe_sol in remaining_sol_set
      self.hint = hint
      self.remaining_sol_set = remaining_sol_set
      self.remaining_cnt = len(remaining_sol_set)

class GuessSet:
   def __init__(self, guess_str: str, num_sols: int, is_in_tot_sol_set: bool, is_in_remaining_sol_set: bool):
      self.guess_str = guess_str
      self.is_in_tot_sol_set = is_in_tot_sol_set
      self.is_in_remaining_sol_set = is_in_remaining_sol_set
      self.guesses = []
      self.num_sols = num_sols
      self.remaining_cnt = sys.maxsize
      self.remaining_avg = float('inf')

   def add_guess(self, guess: Guess):
      self.guesses.append(guess)
      if self.remaining_cnt == sys.maxsize:
         self.remaining_cnt = 0
      self.remaining_cnt += len(guess.remaining_sol_set)
      self.remaining_avg = self.remaining_cnt / self.num_sols
      
   def to_str(self) -> str:
      guess_type = "*"
      if self.is_in_remaining_sol_set:
         guess_type = "+"
      elif self.is_in_tot_sol_set:
         guess_type = "" 
      return f"{self.guess_str}{guess_type}"
   
   def is_better_than(self, other: "GuessSet") -> bool:
      if self.remaining_cnt < other.remaining_cnt:
         return True
      elif self.guess_str != other.guess_str:
         if self.remaining_cnt == other.remaining_cnt:
            #print(f"   Guess {self.to_str()} TIED with {other.to_str()}")

            # Prefer guesses from the remaining solutions when two guesses have the same score
            if self.is_in_remaining_sol_set:
               if not other.is_in_remaining_sol_set:
                  return True
               
            # Prefer guesses from the solutions file when two guesses have the same score
            if self.is_in_tot_sol_set:
               if not other.is_in_tot_sol_set:
                  return True
      return False
