#!/usr/bin/env python3
"""Enforce the normative line and branch thresholds against one LCOV report."""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class CoverageMetric:
    found: int
    hit: int

    @property
    def percent(self) -> float:
        if self.found == 0:
            raise ValueError("coverage report contains no instrumented items")
        return 100.0 * self.hit / self.found


def read_lcov(path: Path) -> tuple[CoverageMetric, CoverageMetric]:
    totals = {"LF": 0, "LH": 0, "BRF": 0, "BRH": 0}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        key, separator, value = raw_line.partition(":")
        if separator and key in totals:
            totals[key] += int(value)
    return (
        CoverageMetric(totals["LF"], totals["LH"]),
        CoverageMetric(totals["BRF"], totals["BRH"]),
    )


def read_thresholds(path: Path) -> tuple[float, float]:
    document = json.loads(path.read_text(encoding="utf-8"))
    thresholds = document["thresholds"]
    return (
        float(thresholds["lineCoveragePercent"]),
        float(thresholds["branchCoveragePercent"]),
    )


def evaluate(lcov_path: Path, quality_plan_path: Path) -> tuple[str, bool]:
    lines, branches = read_lcov(lcov_path)
    minimum_lines, minimum_branches = read_thresholds(quality_plan_path)
    report = (
        f"COVERAGE: lines {lines.hit}/{lines.found} ({lines.percent:.2f}%, "
        f"minimum {minimum_lines:.2f}%); branches {branches.hit}/{branches.found} "
        f"({branches.percent:.2f}%, minimum {minimum_branches:.2f}%)"
    )
    return (
        report,
        lines.percent >= minimum_lines and branches.percent >= minimum_branches,
    )


def main(arguments: list[str]) -> int:
    if len(arguments) != 3:
        print(
            "usage: check_lcov_thresholds.py <lcov.info> <quality-plan.json>",
            file=sys.stderr,
        )
        return 2
    try:
        report, passed = evaluate(Path(arguments[1]), Path(arguments[2]))
    except (KeyError, OSError, ValueError, json.JSONDecodeError) as error:
        print(f"COVERAGE: invalid evidence: {error}", file=sys.stderr)
        return 2
    print(report)
    if passed:
        print("COVERAGE THRESHOLDS: PASS")
        return 0
    print("COVERAGE THRESHOLDS: FAIL", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
