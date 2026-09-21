#!/usr/bin/env python3
"""Create the contributor backlog on GitHub from structured JSON files.

The backlog lives in ``tools/backlog/*.json``. Each issue is authored with the
required sections (problem, why, scope, guidance, acceptance, tests, security,
dependencies, complexity, definition of done). This tool renders each issue into
a consistent Markdown body and creates it with ``gh``.

Safety:
  * Defaults to ``--dry-run``; nothing is created unless ``--apply`` is passed.
  * Idempotent: issues whose title already exists (open or closed) are skipped.
  * Requires the ``gh`` CLI to be authenticated.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

BACKLOG_DIR = Path(__file__).resolve().parent / "backlog"


def run(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(args, capture_output=True, text=True, encoding="utf-8")


def load_json(path: Path):
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def bullet(items) -> str:
    if not items:
        return "_None._"
    if isinstance(items, str):
        return items
    return "\n".join(f"- {item}" for item in items)


def render_body(issue: dict) -> str:
    return "\n".join(
        [
            "## Problem",
            issue["problem"],
            "",
            "## Why it matters",
            issue["why"],
            "",
            "## Scope",
            bullet(issue.get("scope")),
            "",
            "## Technical guidance",
            bullet(issue.get("guidance")),
            "",
            "## Acceptance criteria",
            bullet(issue.get("acceptance")),
            "",
            "## Tests",
            bullet(issue.get("tests")),
            "",
            "## Security considerations",
            issue.get("security", "None beyond the project-wide guarantees."),
            "",
            "## Dependencies",
            issue.get("deps", "None."),
            "",
            "## Complexity",
            f"`difficulty/{issue.get('complexity', 'medium')}`",
            "",
            "## Definition of done",
            issue.get("dod", "Merged with tests passing and CI green."),
            "",
        ]
    )


def existing_titles() -> set[str]:
    result = run(
        [
            "gh",
            "issue",
            "list",
            "--state",
            "all",
            "--limit",
            "2000",
            "--json",
            "title",
        ]
    )
    if result.returncode != 0:
        print("Failed to list existing issues:", result.stderr, file=sys.stderr)
        sys.exit(1)
    return {item["title"] for item in json.loads(result.stdout or "[]")}


def existing_labels() -> set[str]:
    result = run(["gh", "label", "list", "--limit", "500", "--json", "name"])
    if result.returncode != 0:
        return set()
    return {item["name"] for item in json.loads(result.stdout or "[]")}


def create_labels(labels, apply: bool) -> None:
    existing = existing_labels()
    for label in labels:
        name = label["name"]
        if name in existing:
            continue
        if not apply:
            print(f"[dry-run] label {name}")
            continue
        result = run(
            [
                "gh",
                "label",
                "create",
                name,
                "--color",
                label.get("color", "ededed"),
                "--description",
                label.get("description", ""),
            ]
        )
        if result.returncode != 0:
            print(f"  (label {name}: {result.stderr.strip()})")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="actually create issues")
    parser.add_argument("--limit", type=int, default=0, help="max issues to create")
    args = parser.parse_args()

    labels = load_json(BACKLOG_DIR / "labels.json")
    issue_files = sorted(p for p in BACKLOG_DIR.glob("*.json") if p.name != "labels.json")

    issues: list[dict] = []
    for path in issue_files:
        for issue in load_json(path):
            issue.setdefault("_source", path.name)
            issues.append(issue)

    titles = [i["title"] for i in issues]
    duplicates = {t for t in titles if titles.count(t) > 1}
    if duplicates:
        print("Duplicate issue titles in backlog:", duplicates, file=sys.stderr)
        return 1

    print(f"Backlog: {len(issues)} issues across {len(issue_files)} files.")
    create_labels(labels, args.apply)

    existing = existing_titles()
    to_create = [i for i in issues if i["title"] not in existing]
    skipped = len(issues) - len(to_create)
    if args.limit:
        to_create = to_create[: args.limit]

    print(f"Already present: {skipped}. To create: {len(to_create)}.")
    if not args.apply:
        for issue in to_create:
            print(f"[dry-run] {issue['_source']}: {issue['title']}")
        print("Dry run complete. Re-run with --apply to create.")
        return 0

    created = 0
    for issue in to_create:
        body = render_body(issue)
        with tempfile.NamedTemporaryFile(
            "w", suffix=".md", delete=False, encoding="utf-8"
        ) as handle:
            handle.write(body)
            body_path = handle.name
        cmd = ["gh", "issue", "create", "--title", issue["title"], "--body-file", body_path]
        for label in issue.get("labels", []):
            cmd += ["--label", label]
        result = run(cmd)
        Path(body_path).unlink(missing_ok=True)
        if result.returncode != 0:
            print(f"FAILED: {issue['title']}\n  {result.stderr.strip()}", file=sys.stderr)
            continue
        created += 1

    print(f"Created {created} issue(s).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
