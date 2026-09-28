#!/usr/bin/env python3
"""End-to-end CLI benchmark for `soroban-scan`.

The in-process `benchmark` example (see docs/development/benchmarks.md) measures
parsing and detection on synthetic sources. This script measures the *combined*
system end to end — project discovery, parsing, scanning and report generation —
by timing the real release binary over a target directory.

It is dependency-free (standard library only) and deterministic in what it
measures: the same target, the same number of runs, the same profile.

Usage:
    python tools/benchmark.py --target examples/vulnerable-contract --runs 10

The release binary is built automatically if it is missing.
"""

from __future__ import annotations

import argparse
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "target" / "release" / (
    "soroban-scan.exe" if sys.platform.startswith("win") else "soroban-scan"
)
OUT = ROOT / "target" / "benchmark-report.json"


def ensure_binary() -> None:
    if BIN.exists():
        return
    subprocess.run(
        ["cargo", "build", "--release", "-p", "soroban-scan-cli"],
        cwd=ROOT,
        check=True,
    )


def run_once(target: Path) -> float:
    start = time.perf_counter()
    subprocess.run(
        [
            str(BIN),
            "scan",
            str(target),
            "--format",
            "json",
            "--output",
            str(OUT),
            "--fail-on",
            "none",
            "--quiet",
        ],
        cwd=ROOT,
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return time.perf_counter() - start


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--target",
        default="examples/vulnerable-contract",
        help="directory to scan, relative to the repository root",
    )
    parser.add_argument(
        "--runs", type=int, default=10, help="measured runs after a warm-up"
    )
    args = parser.parse_args()

    target = (ROOT / args.target).resolve()
    if not target.exists():
        print(f"error: target not found: {target}", file=sys.stderr)
        return 2
    if args.runs < 1:
        print("error: --runs must be >= 1", file=sys.stderr)
        return 2

    ensure_binary()
    run_once(target)  # warm-up
    samples = sorted(run_once(target) * 1000 for _ in range(args.runs))

    print(f"target: {target.relative_to(ROOT)}")
    print(f"runs:   {args.runs}")
    print(f"min:    {samples[0]:.1f} ms")
    print(f"median: {statistics.median(samples):.1f} ms")
    print(f"mean:   {statistics.mean(samples):.1f} ms")
    print(f"max:    {samples[-1]:.1f} ms")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
