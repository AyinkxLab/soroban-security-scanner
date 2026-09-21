# Contributing through Drips Wave

This project participates in open-source contribution waves. This page explains
how work is scoped, how to claim an issue, and what to expect from maintainers.

## Complexity and points

Every issue carries exactly one complexity label, which maps directly to the
points used by the wave:

| Label | Points | Meaning |
| ----- | ------ | ------- |
| `difficulty/trivial` | 100 | Small, clearly bounded change with obvious acceptance criteria. |
| `difficulty/medium` | 150 | A standard feature or logic touching several parts of the codebase. |
| `difficulty/high` | 200 | Complex engineering: integrations or architectural changes. |

Complexity is assigned honestly. Points are not inflated for easy work and hard
work is not underpriced. If an issue is too large for a single cycle, it is split
into smaller independent issues rather than relabelled.

## Claiming an issue

1. Pick an issue. Start with [`good-first-issue`](https://github.com/AyinkxLab/soroban-security-scanner/labels/good-first-issue)
   if you are new.
2. **Request assignment before starting.** Comment on the issue saying you would
   like to work on it. Do not open a pull request for an issue you have not been
   assigned.
3. Wait for a maintainer to assign you. This avoids duplicate work.

## Working on an issue

1. Fork the repository and create a branch named for the change, for example
   `feat/cli-json-schema` or `fix/parser-crlf`.
2. Read the issue's **Scope**, **Technical guidance**, and **Acceptance
   criteria** sections; they define what "done" means.
3. Write tests alongside the change.
4. Run the local gate:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all
   ```

5. Open a pull request whose description includes `Closes #<issue-number>` and a
   short summary of how you verified the change.

## What maintainers provide

- An initial response to new issues and pull requests within **48 hours**.
- A review decision (approve, request changes, or close with explanation) within
  **72 hours** of a reviewable pull request.
- Clear, actionable feedback, and a merge once the acceptance criteria are met.
- Honest complexity and scope on every issue.

If a maintainer is unavailable, a note is posted on the issue rather than
leaving contributors waiting.

## Security

Do not report or discuss a security vulnerability in a public issue or pull
request. Follow [`SECURITY.md`](../SECURITY.md) and use private reporting.

## Definition of done

An issue is complete when:

- The change is implemented and matches the acceptance criteria.
- Tests are added and the full local gate passes.
- Documentation and `CHANGELOG.md` are updated when behavior is user-visible.
- The pull request is reviewed and merged.

## Useful references

- [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- [`docs/development/setup.md`](development/setup.md)
- [`docs/contributors/writing-rules.md`](contributors/writing-rules.md)
- [`docs/ISSUE_BACKLOG.md`](ISSUE_BACKLOG.md)
