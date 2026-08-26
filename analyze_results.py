#!/usr/bin/env python3
"""Analyze Wordle simulator result files."""

import argparse
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path


RESULT_ROW = re.compile(r"^\s*([A-Za-z]{5}),\s*(\d+)(?:,|\s*$)")
COUNT = re.compile(r"\((\d+)\)")


@dataclass
class Result:
    solution: str
    steps: int
    remaining: list[tuple[int, int]]


def read_results(path: Path) -> tuple[int, list[Result]]:
    results = []
    lines = path.read_text().splitlines()
    for line in lines:
        match = RESULT_ROW.match(line)
        if not match:
            continue

        counts = [int(count.group(1)) for count in COUNT.finditer(line)]
        results.append(Result(match.group(1).upper(), int(match.group(2)), list(enumerate(counts, 1))))

    if not results:
        raise ValueError(f"No result rows found in {path}")
    return len(lines), results


def print_analysis(line_count: int, results: list[Result], path: Path) -> None:
    total = len(results)
    average_steps = sum(result.steps for result in results) / total
    by_steps = Counter(result.steps for result in results)
    groups = defaultdict(lambda: {"occurrences": 0, "solutions": set(), "steps": []})

    for result in results:
        for step, remaining in result.remaining:
            group = groups[remaining]
            group["occurrences"] += 1
            group["solutions"].add(result.solution)
            group["steps"].append(step)

    print(f"File: {path}")
    print(f"Total lines: {line_count}")
    print(f"Result lines: {total}")
    duplicates = [solution for solution, count in Counter(result.solution for result in results).items() if count > 1]
    if duplicates:
        print(f"Duplicate result solutions: {', '.join(sorted(duplicates))}")
    print(f"Average steps: {average_steps:.2f}")
    print()
    print("Results grouped by steps:")
    print("  Steps   Solutions   Percent")
    for steps in sorted(by_steps):
        count = by_steps[steps]
        print(f"  {steps:>5}   {count:>9}   {100 * count / total:>6.2f}%")
    print(f"  {'Total':>5}   {total:>9}   {100.0:>6.2f}%")

    print()
    print("Groups by remaining solutions:")
    print("  Remaining   Occurrences   Solutions   Avg. step   Percent of results")
    for remaining in sorted(groups):
        group = groups[remaining]
        average_step = sum(group["steps"]) / len(group["steps"])
        percent = 100 * len(group["solutions"]) / total
        print(
            f"  {remaining:>9}   {group['occurrences']:>12}"
            f"   {len(group['solutions']):>9}   {average_step:>9.2f}"
            f"   {percent:>17.2f}%"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output_file", type=Path, help="simulator output text file")
    args = parser.parse_args()
    line_count, results = read_results(args.output_file)
    print_analysis(line_count, results, args.output_file)


if __name__ == "__main__":
    main()