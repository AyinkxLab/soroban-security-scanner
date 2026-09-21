# Getting Started

This guide takes you from installation to interpreting a scan.

## 1. Install

See [Installation](installation.md). Confirm it works:

```bash
soroban-scan version
```

## 2. Scan a project

```bash
soroban-scan scan ./my-soroban-contract
```

The scanner discovers Rust source, identifies Soroban contracts, runs the enabled
rules, and prints findings grouped by severity:

```
soroban-scan 0.1.0
Project: ./my-soroban-contract (confirmed_soroban)
Files: 4  Rules run: 7  Skipped: 0

HIGH     SS-001  src/lib.rs:12:9
         State-changing entry point without caller authorization
         confidence: medium
         evidence: `set` storage mutation with no authorization in `set_admin`: …
         fix: Call `address.require_auth()` …

1 finding(s) — critical: 0, high: 1, medium: 0, low: 0, info: 0
```

Exit codes: `0` clean, `1` findings at/above the failure threshold (`high` by
default), `2` usage error, `3` runtime error.

## 3. Understand a finding

Every finding cites a file, line, column, evidence, and remediation. For the full
rule description:

```bash
soroban-scan explain SS-001
```

## 4. Explore the rules

```bash
soroban-scan rules
soroban-scan rules --search storage
soroban-scan rules --category authorization
```

## 5. Choose an output format

```bash
soroban-scan scan . --format json --output report.json --fail-on none
soroban-scan scan . --format sarif --output results.sarif
soroban-scan scan . --format markdown --output report.md
```

## 6. Configure

Create a starter configuration:

```bash
soroban-scan init
```

```toml
min_severity = "low"
exclude = ["vendor/**"]

[rules]
disabled = ["SS-006"]
```

Validate it:

```bash
soroban-scan config validate
```

See [Configuration](configuration.md).

## 7. Add a baseline

Accept current findings and fail only on new ones:

```bash
soroban-scan scan . --write-baseline .soroban-scan-baseline.json
soroban-scan scan . --baseline .soroban-scan-baseline.json --fail-on high
```

See [Baselines](baselines.md).

## 8. Use it in CI

```yaml
- run: cargo install --path crates/soroban-scan-cli
- run: soroban-scan scan ./contracts --format sarif --output results.sarif --fail-on high
- uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```

See [GitHub Integration](github-integration.md).

## Next steps

- [CLI Reference](cli.md)
- [Rule catalog](rules/README.md)
- [Security model](security/security-model.md)
