# Contributor Roadmap

This roadmap is **derived from real remaining work** in the implemented product.
It is populated only after the corresponding functionality exists or is fully
specified. Issues are tracked on GitHub.

> Guiding rule: **quality over quantity.** No artificial issues. No issue padding.
> If the product genuinely supports 150 meaningful tasks, there are 150; if it
> supports 300, there are 300.

## How work is organized

- Every issue describes: problem, why it matters, context, scope, technical
  guidance, acceptance criteria, tests, security considerations, dependencies,
  complexity, and definition of done.
- Complexity uses an honest three-level scale: `difficulty/trivial`,
  `difficulty/medium`, `difficulty/high`.
- Issues are sized to be completable in a single contribution cycle. Oversized
  work is split into independent, meaningful pieces; trivial work is merged into
  related tasks.

## Categories

- Core scanner
- Parser improvements
- Security detectors
- Security fixtures and corpus
- False-positive reduction
- Performance
- CLI and configuration
- Reporting (terminal, JSON, SARIF, Markdown)
- GitHub integration and CI
- Developer tooling
- Documentation
- Testing and fuzzing
- Security research
- AI assistance (optional)
- Accessibility and observability
- Release engineering

## Phases

The implementation proceeds through eleven phases. See
[`../PROJECT_STATUS.md`](../PROJECT_STATUS.md) for current status. Contributor
issues are organized with `phase-N` labels that correspond to these phases.

## Live backlog

The active backlog lives on GitHub as issues labeled `phase-N`, with category
labels (`security`, `soroban`, `rust`, `cli`, `github`, `ci`, `testing`,
`documentation`, `performance`, `developer-experience`, `ai`) and complexity
labels (`difficulty/trivial`, `difficulty/medium`, `difficulty/high`).

The backlog is authored from structured definitions under `tools/backlog/` and
created with `tools/create_backlog.py` (dry-run by default, idempotent). This
keeps issue quality consistent and avoids duplicate or artificial issues.

Start with `good-first-issue` if you are new.

## Adding roadmap items

Propose new work by opening a feature request. A roadmap item is accepted when it
improves at least one of: product functionality, developer experience, security,
reliability, performance, documentation, testing, maintainability, or ecosystem
integration.
