EXCLUSION_CHAR = "^"
UNKNOWN_CHAR = "_"

FILTER_LENGTH = 5
EMPTY_FILTER = UNKNOWN_CHAR * FILTER_LENGTH

class Filter:
   def __init__(self, include_str: str, exclude_str: str):
      
      # Sanity check include string
      stripped_include = include_str.replace(UNKNOWN_CHAR, "")
      include_len = len(stripped_include)
      if include_len > 0:
         assert stripped_include.isalpha(), f"Filter include string '{include_str}' is not alphabetic"
      
      include_list = list(include_str)
      while len(include_list) < FILTER_LENGTH:
         include_list.append(UNKNOWN_CHAR)
      assert len(include_list) == FILTER_LENGTH, f"Filter include string '{include_str}' is not 5 characters long"

      # Sanity check exclude string
      stripped_exclude = exclude_str.replace(UNKNOWN_CHAR, "")
      exclude_len = len(stripped_exclude)
      if exclude_len > 0:
         assert stripped_exclude.isalpha(), f"Filter exclude string '{exclude_str}' is not alphabetic"
      
      exclude_list = list(exclude_str)
      while len(exclude_list) < FILTER_LENGTH:
         exclude_list.append(UNKNOWN_CHAR)
      assert len(exclude_list) == FILTER_LENGTH, f"Filter exclude string '{exclude_str}' is not 5 characters long"

      if include_len > 0 and exclude_len > 0:
         for i, c in enumerate(include_list):
            if c != UNKNOWN_CHAR:
               assert exclude_list[i] == UNKNOWN_CHAR, f"Filter exclude string '{exclude_str}' collides with include '{include_str}'"
            else:
               assert exclude_list[i] != UNKNOWN_CHAR, f"Filter exclude string '{exclude_str}' collides with include '{include_str}'"

      self.include = include_list
      self.exclude = exclude_list
      self.is_empty = include_len==0 and exclude_len==0

   @classmethod
   def make_filter(cls, filter_str: str):
      filter_split_list = filter_str.split(EXCLUSION_CHAR)

      if len(filter_split_list) == 2:
         return Filter(filter_split_list[0], filter_split_list[1])
      elif len(filter_split_list) == 1:
         return Filter(filter_split_list[0], EMPTY_FILTER)
      else:
         raise ValueError(f"Invalid filter string {filter_str}")

   @classmethod
   def make_filter_func(cls, filter_str: str):
      filter = Filter.make_filter(filter_str)

      def filter_func(maybe_sol: str) -> bool:
         assert maybe_sol.isalpha(), f"Possible solution {maybe_sol} is not alphabetic"
         assert maybe_sol.upper() == maybe_sol, f"Possible solution {maybe_sol} is not uppercase"
         assert len(maybe_sol) == 5, f"Possible solution {maybe_sol} is not 5 characters long"

         sol_list = list(maybe_sol)

         # Loop through the filter known positions
         for i, include_char in enumerate(filter.include):
            if include_char != UNKNOWN_CHAR and include_char.isupper():
               if sol_list[i] != include_char:
                  return False
               sol_list[i] = UNKNOWN_CHAR

         # Loop through the filter unknown positions
         for i, include_char in enumerate(filter.include):
            if include_char != UNKNOWN_CHAR and include_char.islower():
               up_ch = include_char.upper()
               if sol_list[i] == up_ch:
                  return False
             
               try:
                  pos = sol_list.index(up_ch)
               except:
                  return False
               sol_list[pos] = UNKNOWN_CHAR

         # Loop through the filter exclusions
         for i, exclude_char in enumerate(filter.exclude):
            if exclude_char != UNKNOWN_CHAR:
               up_ch = exclude_char.upper()
               if up_ch in sol_list:
                  return False

         return True
      return filter_func
   
   @classmethod
   def make_hint(cls, guess: str, solution: str) -> "Filter":
      sol = list(solution)
      # Make a hint based on the guess and the solution
      include = list(EMPTY_FILTER)
      exclude = list(EMPTY_FILTER)
      for i, c in enumerate(guess):
         if c == sol[i]:
            include[i] = c.upper()
            sol[i] = UNKNOWN_CHAR

      for i, c in enumerate(guess):
         try:
            pos = sol.index(c)
         except ValueError:
            if include[i] != c:
               exclude[i] = c.lower()
            continue
         include[i] = c.lower()
         sol[pos] = UNKNOWN_CHAR
 
      return Filter("".join(include), "".join(exclude))

   def __str__(self) -> str:
      filter_str = "".join(self.include)
      exclude_str = "".join(self.exclude)
      if exclude_str != EMPTY_FILTER:
         filter_str += EXCLUSION_CHAR + exclude_str
      return filter_str
   
   def is_match(self, guess_str: str) -> bool:
      return ("".join(self.include) == guess_str) and ("".join(self.exclude) == EMPTY_FILTER)
      
   @classmethod
   def unit_test_hint(cls):
      # Test the make_hint function
      test_cases = [
         ("ABACK", "BREAD", "ab___^__ack"),
         ("ABACK", "BREAK", "ab__K^__ac_"),
         ("ABACK", "BREAM", "ab___^__ack"),
         
         ("ABAMK", "BREAD", "ab___^__amk"),
         ("ABAMK", "BREAK", "ab__K^__am_"),
         ("ABAMK", "BREAM", "ab_m_^__a_k"),
      ]
      for guess, solution, expected in test_cases:
         hint = cls.make_hint(guess, solution)
         
         assert str(hint) == expected, f"Make hint {solution} failed for guess {guess}. Expected {expected}, got {hint}"

   @classmethod
   def unit_test_filter(cls):
      # Test the filter function
      test_cases = [
         ("BAC__", "BACON", True),
         
         ("_ALE", "VALET", True),
         ("_ALE", "PALER", True),
         ("_ALE", "BALER", True),
         ("_ALE", "PALED", True),
         
         ("_ALEb", "BALER", True),
         ("_ALEb", "BALED", True),
         
         ("_alek", "SALEK", False),
         ("_alek", "ALIKE", True),
         ("_alek", "ANKLE", True),
         ("_alek", "FLAKE", True),
         ("_alek", "LEAKY", True),
         ("_alek", "SLAKE", True),
         
         ("_kela", "SALEK", True),
         ("_kela", "ALIKE", True),
         ("_kela", "FLAKE", True),
         ("_kela", "LEAKY", True),
         ("_kela", "SLAKE", True),
         ("_kela", "LATKE", True),
         
         ("Alek", 'ANKLE', True),
         
         ("ab___^__ack", "BREAD", True),
         ("ab__K^__ac_", "BREAK", True),
         ("ab___^__ack", "BREAM", True),
         ("ab___^__ack", "ABACK", False),
         ("ab__K^__ac_", "ABACK", False),
         ("ab___^__ack", "ABACK", False),
                   
         ("ab___^__amk", "BREAD", True),
         ("ab__K^__am_", "BREAK", True),
         ("ab_m_^__a_k", "BREAM", True),
         
         ("BREA_^____d", "BREAD", False),
         ("BREA_^____d", "BREAK", True),
         ("BREA_^____d", "BREAM", True),
      ]
      for filter, candidate, expected in test_cases:
         filter_func = cls.make_filter_func(filter)
         result = filter_func(candidate)
         
         assert result == expected, f"Filter {filter} failed for candidate {candidate}. Expected {expected}, got {result}"

# Run unit tests
Filter.unit_test_filter()
Filter.unit_test_hint()
