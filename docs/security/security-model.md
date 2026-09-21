# Security Model

This document states the guarantees the scanner makes and, equally important, the
guarantees it does **not** make.

## What the scanner does

- Reads source files and manifests from a target directory.
- Parses Rust source statically.
- Applies deterministic rules and emits findings with evidence.
- Writes reports to stdout or to user-specified files.
- Optionally reads a baseline file to classify findings as new/resolved.

## What the scanner never does

- Never requests, stores, or uses private keys or wallet credentials.
- Never signs or submits transactions.
- Never deploys contracts.
- Never executes scanned code or contract binaries.
- Never runs `cargo build`/`cargo run` on the target.
- Never installs dependencies or downloads executables.
- Never makes network requests by default.
- Never modifies the scanned repository.
- Never emits secrets or environment variables in reports.
- Never fabricates findings, source locations, or certainty.

## Treating input as hostile

Scanned repositories are untrusted. The scanner:

- Does not follow symlinks outside the scan root by default.
- Applies exclude rules and a scan root boundary.
- Treats repository-provided configuration as untrusted data that cannot grant
  capabilities.
- Handles parser failures gracefully without panicking.
- Bounds resource usage on pathologically large or deeply nested input.

## Network policy

The default build performs **no network access**. If network-dependent features
are introduced later (for example, optional AI assistance), they must:

- Be explicitly opt-in and disabled by default.
- Validate URLs and enforce an allowlist.
- Enforce timeouts and response-size limits.
- Protect against SSRF and unsafe redirects.
- Fail closed on error.
- Never send source code or secrets off-device without explicit, informed consent.

## AI policy

AI is optional and advisory. The deterministic engine remains authoritative.

AI must never:

- Invent findings or source locations.
- Override or suppress deterministic findings.
- Claim certainty without evidence.
- Access secrets or execute actions.
- Modify code automatically.

AI-generated text is always clearly labeled as AI-generated.

## Reporting secret exposure

If the scanner itself leaks a secret or environment variable in its output, treat
it as a security vulnerability and report it per [`../../SECURITY.md`](../../SECURITY.md).
