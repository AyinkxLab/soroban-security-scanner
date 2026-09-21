# Upgrading

## General guidance

- Read [`CHANGELOG.md`](../../CHANGELOG.md) before upgrading.
- Pin the version you depend on in CI, and upgrade deliberately.
- Run `soroban-scan config validate` after upgrading to catch configuration
  changes early.

## Breaking-change policy

- **Rule ids are never reused or renumbered.** A retired rule keeps its id and is
  marked deprecated.
- **Exit codes are stable.** Documented codes will not change meaning.
- **SARIF and baseline formats are versioned.** A baseline written by an older
  version remains readable within the same `version`; unsupported versions are
  rejected with a clear error.
- **JSON report shape is additive** within a minor line.
- **Severity or confidence of a rule may change** as detectors improve. This is
  announced in the changelog, and `severity_overrides` / `confidence_overrides`
  let you pin behavior you depend on.

## Baselines across upgrades

Fingerprints are derived from rule id, file, and evidence, not line numbers, so
baselines survive refactors and upgrades. If a rule's evidence string changes,
its fingerprint changes and the finding appears as new; re-run
`--write-baseline` after reviewing.

## Configuration across upgrades

Unknown fields are rejected. If a new version adds configuration fields, existing
files keep working because fields are optional with defaults. If a field is
renamed or removed, the changelog and release notes call it out.

## Downgrading

Downgrading is not formally supported. Baselines and configs written by a newer
version may be rejected by an older one.

## Migrating from other tools

- **`clippy`** — complementary. Keep using it; the scanner focuses on Soroban
  semantics.
- **`cargo-audit` / `cargo-deny`** — complementary. Use them for dependency
  advisories and the scanner for contract logic.
- **Finding equivalence** — map another tool's findings to `SS-###` rule ids and
  suppress the scanner's equivalents for accepted issues via
  `[rules] disabled` if you prefer a single source of truth.
