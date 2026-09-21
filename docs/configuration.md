# Configuration

The scanner reads configuration from `soroban-scan.toml` in the scan root or the
current directory, unless `--config` selects a specific file. Command-line
options override the file.

Generate a starter file:

```bash
soroban-scan init
```

Inspect the effective configuration and validate it:

```bash
soroban-scan config show
soroban-scan config validate
```

## Reference

```toml
# Minimum severity to report: info, low, medium, high, critical
min_severity = "info"

# Minimum confidence to report: low, medium, high
min_confidence = "low"

# Glob patterns (relative to the scan root) to exclude from discovery
exclude = ["vendor/**", "target/**"]

# Follow symbolic links during discovery (disabled by default for safety)
follow_symlinks = false

[rules]
# Rule ids to disable
disabled = ["SS-006"]

# If non-empty, only these rule ids run
enabled = []

# Per-rule severity overrides
severity_overrides = { "SS-007" = "low" }

# Per-rule confidence overrides
confidence_overrides = { }

[ai]
# Optional assistance layer. Disabled by default; the deterministic scanner is
# authoritative. See docs/ai.md.
enabled = false
# provider = "openai"
# model = "..."
```

### Fields

| Field | Type | Default | Meaning |
| ----- | ---- | ------- | ------- |
| `min_severity` | severity | `info` | Minimum severity to report. |
| `min_confidence` | confidence | `low` | Minimum confidence to report. |
| `exclude` | array of globs | `[]` | Paths excluded from discovery. |
| `follow_symlinks` | bool | `false` | Follow symlinks (off by default for safety). |
| `rules.disabled` | array of ids | `[]` | Rules never run. |
| `rules.enabled` | array of ids | `[]` | If non-empty, the only rules that run. |
| `rules.severity_overrides` | map id→severity | `{}` | Replace a rule's default severity. |
| `rules.confidence_overrides` | map id→confidence | `{}` | Replace a rule's default confidence. |
| `ai.enabled` | bool | `false` | Enable the optional assistance layer (fails closed without a provider). |
| `ai.provider` | string | *(none)* | Provider name; none is bundled. |
| `ai.model` | string | *(none)* | Model identifier. |

Unknown fields are rejected, as are references to unknown rule ids.

## Precedence

1. Built-in defaults.
2. Configuration file (`--config`, scan root, or current directory).
3. Command-line flags (`--min-severity`, `--min-confidence`, `--exclude`,
   `--disable`, `--rule`).

## Example: strict CI policy

```toml
min_severity = "medium"
min_confidence = "medium"
exclude = ["tests/**", "examples/**"]

[rules]
disabled = ["SS-006"]
severity_overrides = { "SS-002" = "high" }
```

Combined with `--fail-on high`, medium findings are reported but only high and
critical findings fail the build.

## Security

Configuration files are untrusted data. No option can enable code execution,
network access, or access to secrets. See
[Security Model](security/security-model.md).
