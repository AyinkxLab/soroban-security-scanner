# Baselines and Incremental Scanning

A baseline records findings you have reviewed and accepted. New scans then report
which findings are **new** versus already known, so CI can gate on regressions
without failing on pre-existing issues.

## Creating a baseline

```bash
soroban-scan scan ./contracts --write-baseline .soroban-scan-baseline.json
```

This writes the current findings' fingerprints to the baseline file and exits
successfully.

## Scanning against a baseline

```bash
soroban-scan scan ./contracts --baseline .soroban-scan-baseline.json
```

The terminal summary reports `new`, `existing`, and `resolved` counts. When a
baseline is supplied, the `--fail-on` gate considers **only new findings**, so
known issues do not fail CI.

Use `--new-only` to print only new findings:

```bash
soroban-scan scan ./contracts \
  --baseline .soroban-scan-baseline.json \
  --new-only \
  --format json
```

## Incremental scanning

When you scan only a subset of files (for example, changed files in a pull
request), resolved findings cannot be determined, because files that were not
scanned may still contain them. Pass `--incremental` to disable resolved
reporting:

```bash
soroban-scan scan ./changed/src/lib.rs \
  --baseline .soroban-scan-baseline.json \
  --incremental
```

## Fingerprints

Fingerprints are computed from the **rule id**, the **scan-relative file path**,
and the **normalized evidence**. Line numbers are deliberately excluded, so code
movement does not resurrect known findings or create false "new" findings.
Editing the evidence (for example, changing the flagged call) produces a new
fingerprint, which is intentional.

## Baseline file format

```json
{
  "version": 1,
  "tool": "soroban-scan",
  "tool_version": "0.1.0",
  "findings": [
    {
      "fingerprint": "…",
      "rule_id": "SS-001",
      "file": "src/lib.rs",
      "severity": "high",
      "title": "State-changing entry point without caller authorization"
    }
  ]
}
```

Entries are sorted by fingerprint for deterministic output. Baselines are
treated as untrusted input and validated on load; unsupported versions are
rejected.

## Recommended workflow

1. Add an initial baseline to the repository.
2. In CI, scan with `--baseline` and `--fail-on high`.
3. Fix baseline findings over time and refresh the baseline deliberately:

   ```bash
   soroban-scan scan ./contracts --write-baseline .soroban-scan-baseline.json
   ```

4. Review baseline changes in code review, just like any other file.
