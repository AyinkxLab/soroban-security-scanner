# GitHub Integration

The scanner integrates with GitHub through a composite Action, a SARIF workflow,
and changed-file pull-request analysis. All integrations are read-only by default
and never post or create anything unless explicitly enabled.

## Permissions summary

| Capability | Permission | Notes |
| ---------- | ---------- | ----- |
| Read repository contents | `contents: read` | Always required. |
| Upload SARIF | `security-events: write` | Required for code scanning. Not available to fork PRs. |
| Comment on a PR | `pull-requests: write` | Only when opt-in commenting is enabled. |
| Create issues | `issues: write` | Only for explicitly enabled issue creation; not enabled by default. |

The scanner itself needs no token and makes no network requests.

## Using the action

```yaml
name: Security
on: [push, pull_request]

permissions:
  contents: read
  security-events: write

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: AyinkxLab/soroban-security-scanner@main
        with:
          path: ./contracts
          format: sarif
          output: results.sarif
          fail-on: high
      - uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: results.sarif
```

The action builds the scanner from source. Use `if: always()` on the upload step
so results are still published when the gate fails.

### Action inputs

| Input | Default | Description |
| ----- | ------- | ----------- |
| `path` | `.` | Directory or file to scan. |
| `format` | `sarif` | `terminal`, `json`, `sarif`, or `markdown`. |
| `output` | `soroban-scan-results.sarif` | Report path. |
| `fail-on` | `high` | Failure threshold (`none` disables gating). |
| `baseline` | *(empty)* | Baseline file; gate applies to new findings only. |
| `files-from` | *(empty)* | File listing files to scan (changed-file analysis). |
| `args` | *(empty)* | Extra `scan` arguments. |

## SARIF and code scanning

Upload SARIF 2.1.0 to surface findings in the repository's **Security → Code
scanning** tab and as inline annotations on pull requests. Fork pull requests do
not receive `security-events: write`, so upload steps should be guarded:

```yaml
if: github.event.pull_request.head.repo.full_name == github.repository
```

The bundled [`.github/workflows/pr-scan.yml`](../.github/workflows/pr-scan.yml)
does this.

## Changed-file (pull request) analysis

`--files-from` scans only the files listed in a file, which is how the PR
workflow limits work to changed Rust files:

```bash
git diff --name-only --diff-filter=ACMR origin/main...HEAD -- '*.rs' > changed.txt
soroban-scan scan . --files-from changed.txt --baseline .baseline.json --incremental
```

Use `--incremental` so the scanner does not report resolved findings, which
cannot be determined from a partial scan. `--files-from` rejects `..` entries
that would escape the scan root.

## PR annotations

`pr-scan.yml` emits workflow annotations for `high` and `critical` findings using
`::error` commands, independent of code scanning:

```
::error file=src/lib.rs,line=42::SS-001: State-changing entry point without caller authorization
```

## Opt-in PR comments

Commenting is **disabled by default** to avoid noise. Enable it by setting the
repository variable `SOROBAN_SCAN_PR_COMMENT` to `true`:

```bash
gh variable set SOROBAN_SCAN_PR_COMMENT --body true
```

When enabled, the workflow posts a single summary comment per run. It never runs
for fork pull requests.

## Baseline synchronization

Store a baseline in the repository and pass it to the action:

```yaml
- uses: AyinkxLab/soroban-security-scanner@main
  with:
    baseline: .soroban-scan-baseline.json
```

Refresh it deliberately with `soroban-scan scan . --write-baseline …` and review
the change in code review, just like any other file. See
[`baselines.md`](baselines.md).

## Issue creation (opt-in)

The scanner never creates issues automatically. If you want to file issues for
confirmed findings, do it explicitly in your own workflow, gated behind a manual
trigger or a repository variable, for example:

```yaml
- name: File issues for confirmed findings
  if: vars.SOROBAN_SCAN_CREATE_ISSUES == 'true'
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    jq -r '.findings[] | select(.severity == "critical") | "\(.rule_id)\t\(.location.file):\(.location.line)\t\(.title)"' results.json |
    while IFS=$'\t' read -r rule loc title; do
      gh issue create --title "[${rule}] ${title}" --body "Location: \`${loc}\`"
    done
```

Guidance:

- Never enable issue creation for untrusted (fork) pull requests.
- Deduplicate before creating issues; the same fingerprint recurs across runs.
- Keep it manual or scheduled, not on every push.

## Fork pull-request safety

- Fork PRs run with a read-only `GITHUB_TOKEN` and no access to repository
  secrets.
- Never `checkout` untrusted code and run privileged steps together.
- The scanner never executes, builds, or modifies the code it scans, which makes
  it safe to run on fork PR content.
- Guard SARIF upload and commenting with the fork check shown above.

## Security guarantees

- No network access by the scanner.
- No secrets are read or emitted.
- No repository content is modified.
- No issues, comments, or check runs are created unless explicitly enabled.
