# CLI Reference

The `soroban-scan` binary provides deterministic security analysis for Soroban
and Rust projects.

```text
soroban-scan <COMMAND>
```

| Command | Purpose |
| ------- | ------- |
| `scan` | Scan a directory or file for findings. |
| `rules` | List available security rules. |
| `explain <RULE-ID>` | Explain a rule in detail. |
| `version` | Print the scanner name and version. |
| `init` | Write a starter configuration file. |
| `config show` / `config validate` | Inspect or validate configuration. |

---

## `scan`

```text
soroban-scan scan [PATH] [OPTIONS]
```

`PATH` defaults to the current directory.

| Option | Description |
| ------ | ----------- |
| `--format <terminal\|json\|sarif\|markdown>` | Output format (default `terminal`). |
| `--output <FILE>` | Write the report to a file instead of stdout. |
| `--min-severity <LEVEL>` | Only report findings at or above this severity. |
| `--min-confidence <LEVEL>` | Only report findings at or above this confidence. |
| `--exclude <GLOB>` | Exclude a glob relative to the scan root (repeatable). |
| `--config <FILE>` | Use a specific configuration file. |
| `--files-from <FILE>` | Read a newline-separated list of files to scan (relative to `PATH`). |
| `--disable <RULE>` | Disable a rule by id (repeatable). |
| `--rule <RULE>` | Run only the given rule(s) (repeatable). |
| `--fail-on <none\|info\|low\|medium\|high\|critical>` | Failure threshold (default `high`). |
| `--baseline <FILE>` | Compare against a baseline; gate applies to new findings only. |
| `--write-baseline <FILE>` | Write current findings as a baseline and exit. |
| `--new-only` | With `--baseline`, report only new findings. |
| `--incremental` | With `--baseline`, do not report resolved findings. |
| `--guidance` | Print labeled offline guidance for findings (see [ai.md](ai.md)). |
| `--quiet`, `-q` | Compact output; suppresses the header. |
| `--verbose`, `-v` | Include diagnostics. |

### Examples

```bash
# Human-readable scan
soroban-scan scan ./contracts

# JSON for tooling
soroban-scan scan ./contracts --format json --output report.json --fail-on none

# SARIF for GitHub code scanning
soroban-scan scan ./contracts --format sarif --output results.sarif

# Only high and critical findings
soroban-scan scan ./contracts --min-severity high

# A single file
soroban-scan scan ./contracts/src/lib.rs

# Baseline workflow
soroban-scan scan ./contracts --write-baseline .baseline.json
soroban-scan scan ./contracts --baseline .baseline.json --fail-on high
```

See [`baselines.md`](baselines.md) for the baseline workflow, fingerprint
semantics, and incremental scanning.

---

## `rules`

```text
soroban-scan rules [--format terminal|json] [--category CATEGORY] [--min-severity LEVEL] [--search TEXT]
```

Lists the rule catalog. `--search` matches case-insensitively against rule id,
title, description, and category.

## `explain`

```text
soroban-scan explain SS-001 [--format terminal|json|markdown]
```

Prints a rule's description, defaults, remediation, and references.

## `init`

```text
soroban-scan init [PATH] [--force]
```

Writes `soroban-scan.toml`. Refuses to overwrite unless `--force` is given.

## `config`

```text
soroban-scan config show [--config FILE]
soroban-scan config validate [--config FILE]
```

`show` prints the effective configuration as TOML. `validate` checks a
configuration file and verifies that referenced rule ids exist.

---

## Configuration

Configuration is discovered from `soroban-scan.toml` in the scan root or the
current directory, unless `--config` is given. Command-line options override the
file.

```toml
min_severity = "info"
min_confidence = "low"
exclude = ["vendor/**"]
follow_symlinks = false

[rules]
disabled = ["SS-006"]
enabled = []
severity_overrides = { "SS-007" = "low" }
confidence_overrides = { }
```

Unknown fields are rejected. Repository-provided configuration is treated as
untrusted data and cannot enable code execution or network access.

---

## Output formats

- **terminal** — human-readable, grouped listing with remediation.
- **json** — stable structured report (tool, project, stats, findings,
  diagnostics).
- **sarif** — SARIF 2.1.0 for GitHub code scanning and compatible tools.
- **markdown** — report suitable for pull requests and documentation.

## Severity and confidence

Severity: `info`, `low`, `medium`, `high`, `critical`.
Confidence: `low`, `medium`, `high`.

See [`rules/README.md`](rules/README.md) for their exact meaning.

## Exit codes

| Code | Meaning |
| ---- | ------- |
| `0` | Success; no findings at or above the failure threshold. |
| `1` | Findings at or above the failure threshold. |
| `2` | Usage error (bad arguments, unknown rule, invalid configuration). |
| `3` | Runtime error (I/O failure, unreadable scan root). |

`--fail-on` controls the threshold for code `1`. The default is `high`, so
medium/low/info findings do not fail a scan unless configured.

## Security notes

- The scanner never executes, builds, deploys, or modifies scanned code.
- No network access is performed.
- Reports never include environment variables or secrets.
- Paths are relativized to the scan root; traversal outside the root is
  rejected.
