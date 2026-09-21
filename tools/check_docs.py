#!/usr/bin/env python3
"""Check relative Markdown links in the repository.

Read-only and offline. Exits non-zero when a relative link points to a missing
file. External URLs are not fetched.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
SKIP_DIRS = {".git", "target", "node_modules", "vendor"}


def iter_markdown(root: Path):
    for path in root.rglob("*.md"):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        yield path


def is_external(target: str) -> bool:
    return (
        target.startswith("http://")
        or target.startswith("https://")
        or target.startswith("mailto:")
        or target.startswith("#")
        or target.startswith("<")
    )


def check_file(path: Path, root: Path) -> list[str]:
    problems: list[str] = []
    text = path.read_text(encoding="utf-8", errors="replace")
    for match in LINK_RE.finditer(text):
        raw = match.group(1).strip()
        if not raw or is_external(raw):
            continue
        target = raw.split("#", 1)[0].strip()
        if not target:
            continue
        resolved = (path.parent / target).resolve()
        if not resolved.exists():
            problems.append(f"{path.relative_to(root)}: broken link -> {raw}")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="repository root (default: parent of tools/)",
    )
    args = parser.parse_args()
    root = args.root.resolve()

    problems: list[str] = []
    count = 0
    for path in iter_markdown(root):
        count += 1
        problems.extend(check_file(path, root))

    if problems:
        print(f"Found {len(problems)} broken link(s) across {count} files:")
        for problem in problems:
            print(f"  {problem}")
        return 1

    print(f"OK: checked {count} Markdown files, no broken relative links.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
