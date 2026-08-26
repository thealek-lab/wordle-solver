#!/usr/bin/env python3
"""Interactive Wordle solver using feedback-aware filtering and entropy ranking."""

from __future__ import annotations

import math
from optparse import OptionParser
from collections import Counter
from pathlib import Path

DEFAULT_WORDS = """
aback abase abate abbey abbot abide abled abode abort about above abuse actor acute adapt adept admit adobe adopt adore adorn adult after again agent agile aging agony agree ahead alarm album alert algae alias alike alive allow alone along aloud alpha alter amaze amber amend amigo amiss among ample aptly arena argue arise aroma arose aside asset audio audit avail award aware awful bacon badge badly baker basic basil basin basis batch beast began begin being belly below bench berry birth black blade blame blank blast blaze bleak blend bless blind blink block blood bloom blown board boast bonus booth bound boxer brain brake brand brass brave bread break breed brick bride brief bring broad broke brown brush build built buyer cable cafe chain chair chalk champ chant chaos charm chart chase cheap check cheek cheer chess chest chief child chili china choke chose chunk cigar civil claim class clean clear clerk click cliff climb clock close cloth cloud coach coast coral couch could count court cover craft crash crazy cream crime crisp cross crowd crown crude crush curve cycle daily dairy dance dealt death debut delay delta dense depot depth devil diary dirty disco ditch doing doubt dozen draft drama drawn dream dress drift drink drive drove dusty eager early earth easel ebony elbow elder elect elite empty enemy enjoy enter entry equal error essay event every exact exile extra faith false fancy fatal favor feast fence fewer fiber field fifth fifty fight final first flame flash fleet flesh float floor flour focus force forge forth forty forum found frame frank fraud fresh front frost fruit funny giant given glass globe glory glove going grace grade grain grand grant grape graph grasp grass grave great greed green greet grief grill grind gripe gross group grove grown guard guess guest guide guild habit happy harsh haste hasty haunt heart heavy hello hence herbal hero honor horse hotel house human humor ideal image imply inbox index infer inner input intro ivory jewel joint judge juice kneel knife knock known label labor large laser later laugh layer learn lease least leave legal lemon level lever light limit linen liver local lodge logic loose loser lotus lovely lower loyal lucky lunch magic major maker mango mania manor march marry match maybe mayor medal media mercy merge merit metal meter might minor minus model money month moral motor mount mouse mouth movie music naive nasty naval never newly night ninth noble noise noisy north notice novel nurse occur ocean offer often olive onion opera orbit order other ought outer owner oxide ozone paint panel panic paper party pasta patch pause peace peach pearl penny phase phone photo piano piece pilot pinch pipe pitch place plain plane plant plate plead pleat plenty plumb point polar porch pound power press price pride prime print prior prize prove queen query quick quiet quite radio raise rally ranch range rapid ratio reach react ready realm relay reply retry rhyme rider ridge rifle right rival river roast robot rocky rogue rough round route royal rugby ruler rumor rural salad sales salon sauce scale scare scene scent scope score scout scrap screw script scrub seize sense serve seven shade shake shame shape share shark sharp sheep sheet shelf shell shift shine shirt shock shoes shoot shore short shout shown sight since sixth sixty skill skirt slack slain slang sleep slice slide slight slime sling slope smart smear smell smile smoke snack snake sneak sober solar solid solve sound south space spare speak speed spend spent spice spicy spider spike spill spine spire split spoke sport spray squad stack staff stage stain stair stake stale stand stare stark start state steam steel steep steer stem stock stone stood store storm story stove strap straw strip stuck study stuff style sugar suite sunny super swear sweep sweet swell swift swing switch sword syrup table taken taste taught teeth thank their theme there these thick thief thing think third those three threw throw thumb tiger tight timer tired title toast today token topic torch total touch tough towel tower toxic trace track trade trail train trait trash treat trend trial tribe trick tried truck truly trust truth tutor twice uncle under union unite until upper upset urban usage usual valid value vapor vault video virus visit vivid voice voter wagon waist waste watch water weave wedge weigh weird whale wheat wheel where which while white whole whose woman women world worry worse worst worth would wound write wrong yacht yearn yeast young youth zebra"
""".split()

WORDLE_LENGTH = 5

def load_words(path: str | None = None) -> list[str]:
    """Load five-letter words, optionally from a newline-separated file."""
    words = Path(path).read_text().split() if path else DEFAULT_WORDS
    normalized = sorted({word.upper() for word in words if len(word) == WORDLE_LENGTH and word.isalpha()})
    if not normalized:
        raise ValueError("word list contains no five-letter alphabetic words")
    return normalized


def make_feedback(guess: str, answer: str) -> str:
    """Return Wordle feedback: g=green, y=yellow, b=gray."""
    guess, answer = guess.upper(), answer.upper()
    if len(guess) != WORDLE_LENGTH or len(answer) != WORDLE_LENGTH:
        raise ValueError("guess and answer must be five letters")

    result = ["_"] * WORDLE_LENGTH
    remaining = Counter(answer)
    for index, (guess_letter, answer_letter) in enumerate(zip(guess, answer)):
        if guess_letter == answer_letter:
            result[index] = "g"
            remaining[guess_letter] -= 1
    for index, guess_letter in enumerate(guess):
        if result[index] == "_" and remaining[guess_letter] > 0:
            result[index] = "y"
            remaining[guess_letter] -= 1
    return "".join(result)


def matches(candidate: str, guess: str, feedback: str) -> bool:
    """Check whether candidate would produce the supplied feedback."""
    return make_feedback(guess, candidate) == feedback.lower()


def filter_candidates(candidates: list[str], guess: str, feedback: str) -> list[str]:
    feedback = feedback.lower()
    if len(guess) != WORDLE_LENGTH or len(feedback) != WORDLE_LENGTH or any(mark not in "gy_" for mark in feedback):
        raise ValueError("feedback must be five characters using g, y, and _")
    return [candidate for candidate in candidates if matches(candidate, guess, feedback)]


def rank_guesses(guesses: list[str], candidates: list[str], limit: int = 10) -> list[tuple[str, float]]:
    """Rank guesses by expected information, with candidate words preferred on ties."""
    if not candidates:
        return []
    answer_count = len(candidates)
    ranked = []
    for guess in guesses:
        buckets = Counter(make_feedback(guess, answer) for answer in candidates)
        probabilities = [size / answer_count for size in buckets.values()]
        entropy = -sum(probability * math.log2(probability) for probability in probabilities)
        if guess in ["CODEX", "FAZED"]:
            print(f"{guess} {buckets} {entropy} {probabilities}")
        ranked.append((entropy, guess in candidates, guess))
    ranked.sort(key=lambda item: (-item[0], not item[1], item[2]))
    return [(guess, entropy) for entropy, _, guess in ranked[:limit]]


def prompt_feedback(guess: str) -> str:
    while True:
        feedback = input(f"Feedback for {guess} (g/y/_): ").strip().lower()
        if feedback == "q":
            return feedback
        if len(feedback) == WORDLE_LENGTH and set(feedback) <= set("gy_"):
            return feedback
        print("Enter exactly five marks: g (green), y (yellow), or _ (gray).")

def unit_test():
   solution = "FAZED"
   
   test_cases = [("SLATE", "__y_y"),
                 ("RACED", "_g_gg"),
                 ("GAWKY", "_g___"),
                 ("FRUMP", "g____"),
                 ("CODEX", "__yg_")
               ]

   for guess, answer in test_cases:
      score = make_feedback(guess, solution)
      assert score == answer
   
   answer_count = len(test_cases)
   buckets = Counter(make_feedback(guess, solution) for guess, answer in test_cases)
   print(f"buckets {buckets}")
   entropy = -sum((size / answer_count) * math.log2(size / answer_count) for size in buckets.values())
   print(f"entropy {entropy}")

unit_test()

def main() -> None:
    parser = OptionParser(description="Solve Wordle from guesses and color feedback.")
    parser.add_option("--words", help="newline-separated custom five-letter word list")
    parser.add_option("--first", default="slate", help="first guess (default: slate)")
    options, _ = parser.parse_args()

    words = load_words(options.words)
    candidates = words[:]
    guess = options.first.upper()
    if len(guess) != WORDLE_LENGTH or not guess.isalpha():
        parser.error("--first must be five alphabetic letters")

    print(f"Loaded {len(words)} words. Type q at any prompt to quit.")
    while True:
        print(f"\nTry: {guess}    ({len(candidates)} possible answers)")
        feedback = prompt_feedback(guess)
        if feedback == "q":
            return
        if feedback == "ggggg":
            print("Solved.")
            return
        candidates = filter_candidates(candidates, guess, feedback)
        if not candidates:
            print("No candidates remain. Check the guess and feedback.")
            return
        suggestions = rank_guesses(words, candidates)
        print("Best next guesses:", ", ".join(f"{word.upper()} ({score:.2f})" for word, score in suggestions))
        guess = suggestions[0][0]


if __name__ == "__main__":
    main()
