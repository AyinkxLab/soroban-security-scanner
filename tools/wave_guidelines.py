"""Shared Drips Wave contribution guidelines for issue bodies.

Used by ``create_backlog.py`` (when creating issues) and
``apply_wave_guidelines.py`` (when backfilling existing issues).
"""

from __future__ import annotations

REPO_URL = "https://github.com/AyinkxLab/soroban-security-scanner"
MARKER = "## Contribution guidelines"


def render_guidelines(title: str, number: int | None = None) -> str:
    """Render the standard contribution guidelines block for an issue."""
    closes = f"`Closes #{number}`" if number else "`Closes #` followed by this issue's number"
    return "\n".join(
        [
            MARKER,
            "",
            f"- **Assignment required before starting.** Comment on this issue to "
            f"request assignment before writing code.",
            f"- Your pull request must include {closes}.",
            "- Run the local gate before opening the PR:",
            "",
            "  ```",
            "  cargo fmt --all -- --check",
            "  cargo clippy --all-targets --all-features -- -D warnings",
            "  cargo test --all",
            "  ```",
            "",
            f"- Example commit message: `{title}`",
            "- Complexity is tagged with a `difficulty/*` label and maps to wave "
            "points (trivial 100 / medium 150 / high 200).",
            f"- See [docs/WAVE.md]({REPO_URL}/blob/main/docs/WAVE.md) and "
            f"[CONTRIBUTING.md]({REPO_URL}/blob/main/CONTRIBUTING.md).",
            "",
        ]
    )
