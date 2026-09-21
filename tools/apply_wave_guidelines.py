#!/usr/bin/env python3
"""Backfill the standard contribution guidelines onto existing open issues.

Idempotent: issues that already contain the guidelines marker are skipped.
Read-only until it edits issue bodies with ``gh issue edit``.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

from wave_guidelines import MARKER, render_guidelines

PLACEHOLDER = "`Closes #` followed by this issue's number"


def run(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(args, capture_output=True, text=True, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="edit issue bodies")
    parser.add_argument("--limit", type=int, default=0, help="max issues to edit")
    args = parser.parse_args()

    result = run(
        [
            "gh",
            "issue",
            "list",
            "--state",
            "open",
            "--limit",
            "1000",
            "--json",
            "number,title,body",
        ]
    )
    if result.returncode != 0:
        print("Failed to list issues:", result.stderr, file=sys.stderr)
        return 1

    issues = json.loads(result.stdout or "[]")

    def needs_update(issue: dict) -> bool:
        body = issue.get("body") or ""
        return MARKER not in body or PLACEHOLDER in body

    missing = [i for i in issues if needs_update(i)]
    if args.limit:
        missing = missing[: args.limit]

    print(f"Open issues: {len(issues)}. Needing guidelines update: {len(missing)}.")
    if not args.apply:
        for issue in missing:
            print(f"[dry-run] #{issue['number']} {issue['title']}")
        print("Dry run complete. Re-run with --apply to update.")
        return 0

    updated = 0
    for issue in missing:
        body = issue.get("body") or ""
        if MARKER not in body:
            body = body.rstrip() + "\n\n" + render_guidelines(
                issue["title"], issue["number"]
            )
        else:
            body = body.replace(PLACEHOLDER, f"`Closes #{issue['number']}`")
        with tempfile.NamedTemporaryFile(
            "w", suffix=".md", delete=False, encoding="utf-8"
        ) as handle:
            handle.write(body)
            body_path = handle.name
        edit = run(
            ["gh", "issue", "edit", str(issue["number"]), "--body-file", body_path]
        )
        Path(body_path).unlink(missing_ok=True)
        if edit.returncode != 0:
            print(f"FAILED #{issue['number']}: {edit.stderr.strip()}", file=sys.stderr)
            continue
        updated += 1

    print(f"Updated {updated} issue(s).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
