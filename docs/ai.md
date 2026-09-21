# Optional AI Assistance

The scanner's deterministic engine is authoritative. Assistance is an optional,
clearly labeled layer that helps developers understand findings. It is **off by
default**, performs **no network access by default**, and can never change the
deterministic result.

## Guarantees

- The scanner is fully functional with assistance disabled.
- Assistance never creates, removes, suppresses, or overrides findings.
- Assistance never invents source locations or evidence.
- Assistance never claims certainty beyond the observed evidence.
- Assistance never accesses secrets, submits transactions, or modifies code.
- Assistance output is always labeled as assistance.

## Offline guidance (no model, no network)

`--guidance` prints deterministic, rule-based guidance derived solely from the
finding and its rule documentation:

```bash
soroban-scan scan ./contracts --guidance
```

Output is prefixed with a clear label:

```
GUIDANCE (offline, rule-based)

Scanned with 5 deterministic findings: critical 0, high 2, ...
```

This is not model-generated. It is a deterministic restatement of the rule's
description, evidence, and remediation, plus prioritization hints.

## Enabling a model-backed provider

Configuration has an `[ai]` section:

```toml
[ai]
enabled = false
# provider = "openai"
# model = "..."
```

No model provider is bundled. If you set `enabled = true` without a bundled
provider — or name a provider that is not compiled in — the CLI **fails closed**:

```
error: assistance is unavailable: assistance provider 'openai' is not available in this build
```

This is deliberate. Enabling assistance can never silently fall back to
unverified behavior.

## Implementing a provider

A provider implements the `Assistant` trait in
`crates/soroban-scan-core/src/ai/mod.rs`:

```rust
pub trait Assistant {
    fn name(&self) -> &str;
    fn explain_finding(&self, finding: &Finding, rule: Option<&RuleMetadata>) -> Explanation;
    fn summarize(&self, findings: &[Finding]) -> Explanation;
}
```

Requirements for any provider:

1. Set `ai_generated: true` and use [`AI_LABEL`](../crates/soroban-scan-core/src/ai/mod.rs)
   so output is clearly identified as model-generated.
2. Accept only existing findings and rule metadata as input. Never synthesize
   findings or locations.
3. Never include environment variables, secrets, or absolute user paths in
   output.
4. Obey the network policy below.

## Network policy for providers

The default build performs no network access. A provider that needs the network
must:

- Be explicitly opt-in and disabled by default.
- Enforce an allowlist of destinations.
- Enforce request timeouts and response-size limits.
- Validate URLs and reject unsafe redirects.
- Protect against SSRF (no access to internal endpoints or metadata services).
- Fail closed on any error.
- Never send source code or secrets off-device without explicit, informed
  user consent.

## What AI must never do

- Invent findings or source locations.
- Override or suppress deterministic findings.
- Claim certainty without evidence.
- Access secrets or execute actions.
- Modify code automatically.
