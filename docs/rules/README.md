# Security Rules

This document defines the rule system, severity/confidence semantics, and the
process for adding a new Soroban security detector.

## Rule identifiers

Rules use a stable identifier of the form:

```
SS-###   (Soroban Security)
```

Identifiers are **never reused** and **never renumbered**. A retired rule keeps
its identifier and is marked deprecated.

## Severity

Severity describes **potential impact if the issue is real**.

| Severity | Meaning |
| -------- | ------- |
| `critical` | Direct loss or theft of funds, or total loss of access control. |
| `high` | Serious security impact under plausible conditions. |
| `medium` | Security weakness that contributes to an exploit or degrades safety. |
| `low` | Minor weakness or hardening opportunity. |
| `info` | Informational; no direct security impact. |

## Confidence

Confidence describes **how certain the detector is that the finding is real**,
given the evidence it can observe statically.

| Confidence | Meaning |
| ---------- | ------- |
| `high` | Direct, unambiguous evidence in the source. |
| `medium` | Strong evidence, but assumptions or context required. |
| `low` | Heuristic; may be a false positive; requires human review. |

`severity` and `confidence` are independent. A high-severity issue detected only
heuristically is reported as `severity: high, confidence: low`, never
`confidence: high`.

## Categories

`authorization`, `authentication`, `access-control`, `cross-contract`,
`token`, `storage`, `arithmetic`, `error-handling`, `events`, `resource-usage`,
`configuration`, `dependencies`, `unsafe`.

## Built-in detectors

| Rule | Title | Severity | Confidence | Category |
| ---- | ----- | -------- | ---------- | -------- |
| [SS-001](detectors/SS-001.md) | State-changing entry point without caller authorization | high | medium | authorization |
| [SS-002](detectors/SS-002.md) | Cross-contract call without caller authorization | high | low | cross-contract |
| [SS-003](detectors/SS-003.md) | Unbounded storage iteration in an entry point | medium | low | resource-usage |
| [SS-004](detectors/SS-004.md) | Panic-prone construct in a contract entry point | low | medium | error-handling |
| [SS-005](detectors/SS-005.md) | Unsafe code in a Soroban contract | medium | high | unsafe |
| [SS-006](detectors/SS-006.md) | Persistent storage write without TTL management | low | low | storage |
| [SS-007](detectors/SS-007.md) | Hardcoded Stellar account or contract address | medium | low | configuration |

Each detector has a positive and negative fixture under
`fixtures/corpus/<RULE-ID>/`.

## Anatomy of a finding

- `rule_id`, `title`, `description`
- `severity`, `confidence`, `category`
- `file`, `line`, `column` (when available)
- `evidence` (a short, exact excerpt or structural fact)
- `remediation`
- `references`

## Adding a new rule

Before implementing a detector, answer all of the following:

1. **Problem** — what security problem does it address?
2. **Evidence** — what, exactly, is detectable statically?
3. **Limitations** — what can this detector *not* determine?
4. **False positives** — which legitimate patterns must not trigger it?
5. **Positive fixture** — a sample that must trigger.
6. **Negative fixture** — a sample that must not trigger.

Then:

7. Implement the detector against the rule interface.
8. Add unit tests and regression fixtures.
9. Document the rule and its remediation.
10. Set honest `severity` and `confidence`.

Rules must never fabricate evidence or source locations. When evidence is
insufficient, prefer `confidence: low` or do not report at all.

See [`docs/development/testing.md`](../development/testing.md) for the fixture
layout and [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) for the definition of
done.

## Rule documentation template

```markdown
### SS-XXX — <Title>

- **Severity:** <severity>
- **Confidence:** <confidence>
- **Category:** <category>

**Problem.** ...

**How it is detected.** ...

**Limitations.** ...

**False positives.** ...

**Remediation.** ...

**References.** ...
```
