# Threat Model

This document enumerates the threats the scanner is designed to withstand. It is
a living document and is updated as the attack surface changes.

## Assets

- The user's machine and filesystem.
- The scanned repository's confidentiality and integrity.
- The user's credentials and environment variables.
- The integrity of findings (no false confidence, no suppression).
- CI systems where the scanner runs.

## Trust boundaries

| Boundary | Trusted | Notes |
| -------- | ------- | ----- |
| Scanner binary | Trusted | Built from source in CI/release |
| Scanned repository content | **Untrusted** | Hostile input |
| Repository-provided config | **Untrusted** | Cannot grant capabilities |
| Baseline files | **Untrusted** | Parsed defensively |
| CLI flags from the invoking user | Trusted-ish | Validated, no privilege escalation |
| Network | **Untrusted** | Disabled by default |

## Threats and mitigations

### T1 — Malicious source causes parser panic or DoS

**Threat:** crafted source crashes or hangs the scanner.
**Mitigation:** parse defensively; treat parse failures as diagnostics; add
malformed/adversarial input tests; avoid unbounded recursion.

### T2 — Symlink/path traversal escapes the scan root

**Threat:** a symlink causes the scanner to read files outside the target.
**Mitigation:** do not follow symlinks outside the scan root by default; reject
unsafe paths with `ScanError::UnsafePath`.

### T3 — Repository config escalates privileges

**Threat:** a repository ships a config that enables code execution or network
access.
**Mitigation:** config is data, not capability. No config option enables
execution or network access. Unknown/invalid config is rejected.

### T4 — Secret leakage in reports

**Threat:** reports include environment variables, tokens, or absolute user
paths.
**Mitigation:** never read or emit environment variables; relativize paths;
add tests asserting reports contain no `$HOME`/profile paths.

### T5 — Finding suppression / silent failure

**Threat:** errors are hidden and a scan silently reports "clean".
**Mitigation:** scan failures are surfaced; exit codes distinguish success,
findings, and errors; no silent catch-all.

### T6 — Supply-chain compromise

**Threat:** a dependency introduces malicious behavior.
**Mitigation:** minimal, justified dependencies; committed lockfile; `cargo-deny`
checks in CI; review before adding dependencies.

### T7 — SSRF / unsafe network (future opt-in features)

**Threat:** an optional network feature is abused to reach internal services.
**Mitigation:** disabled by default; allowlist; timeout; size limit; redirect
validation; fail closed.

### T8 — Resource exhaustion on very large repositories

**Threat:** scanning a huge repo consumes unbounded memory.
**Mitigation:** streaming traversal; per-file size limits; benchmarks and limits
documented.

## Out of scope

- Maliciousness of findings about third-party code (we report evidence, not
  verdicts).
- Attacks requiring the user to deliberately build a malicious scanner binary.
