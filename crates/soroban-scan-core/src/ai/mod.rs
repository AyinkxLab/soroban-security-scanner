//! Optional assistance layer.
//!
//! Design constraints (non-negotiable):
//!
//! * The deterministic engine is authoritative. Assistance never creates,
//!   removes, or overrides findings.
//! * Assistance is **off by default** and never performs network access on its
//!   own.
//! * Any assistant output is clearly labeled and is derived only from existing
//!   findings and rule metadata. It cannot invent source locations.
//! * Enabling a model-backed provider without a configured, allowlisted provider
//!   fails closed.
//!
//! This module ships an offline, rule-based guidance assistant. A model-backed
//! assistant can be added by implementing [`Assistant`]; it must obey the same
//! constraints and the network policy in `docs/security/security-model.md`.

use serde::{Deserialize, Serialize};

use crate::category::Category;
use crate::finding::Finding;
use crate::rule::RuleMetadata;
use crate::severity::Severity;

/// Label prepended to all offline (non-model) guidance.
pub const OFFLINE_LABEL: &str = "GUIDANCE (offline, rule-based)";

/// Label prepended to model-generated text.
pub const AI_LABEL: &str = "AI-GENERATED ASSISTANCE (not authoritative)";

/// Configuration for the optional assistance layer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AiConfig {
    /// Master switch. Disabled by default.
    pub enabled: bool,
    /// Provider name (for example `openai`). No provider is bundled.
    pub provider: Option<String>,
    /// Model identifier.
    pub model: Option<String>,
}

/// Errors from the assistance layer.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AiError {
    /// Assistance was enabled but no provider is configured.
    #[error("assistance is enabled but no provider is configured")]
    NotConfigured,
    /// A provider was named but is not available in this build.
    #[error("assistance provider '{0}' is not available in this build")]
    ProviderUnavailable(String),
}

/// A clearly labeled piece of assistance text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    /// The text.
    pub text: String,
    /// Whether the text came from a model (true) or from offline rules (false).
    pub ai_generated: bool,
    /// Human-readable label placed before the text.
    pub label: &'static str,
}

impl Explanation {
    /// Renders the label and text.
    pub fn render(&self) -> String {
        format!("{}\n\n{}", self.label, self.text)
    }
}

/// An assistance provider.
///
/// Implementations must never fabricate findings or locations, and must never
/// claim certainty beyond the underlying evidence.
pub trait Assistant {
    /// Provider name.
    fn name(&self) -> &str;

    /// Explains a finding using its existing evidence and rule metadata.
    fn explain_finding(&self, finding: &Finding, rule: Option<&RuleMetadata>) -> Explanation;

    /// Summarizes a scan.
    fn summarize(&self, findings: &[Finding]) -> Explanation;
}

/// Offline, deterministic, rule-based guidance.
#[derive(Debug, Default, Clone, Copy)]
pub struct RuleGuidance;

impl Assistant for RuleGuidance {
    fn name(&self) -> &str {
        "rule-guidance"
    }

    fn explain_finding(&self, finding: &Finding, rule: Option<&RuleMetadata>) -> Explanation {
        let mut text = String::new();
        text.push_str(&format!(
            "{} at {} (severity: {}, confidence: {}).\n\n",
            finding.rule_id,
            finding.location.display_label(),
            finding.severity,
            finding.confidence
        ));
        text.push_str(&format!("What it means: {}\n\n", finding.description));
        text.push_str(&format!("Evidence observed: {}\n\n", finding.evidence));
        text.push_str(&format!(
            "Suggested remediation: {}\n\n",
            finding.remediation
        ));
        if finding.severity >= Severity::High
            && finding.confidence <= crate::confidence::Confidence::Medium
        {
            text.push_str(
                "Note: this is a high-severity, heuristic finding. Review the code \
before treating it as a confirmed vulnerability.\n\n",
            );
        }
        if let Some(rule) = rule {
            if !rule.references.is_empty() {
                text.push_str("References:\n");
                for reference in rule.references {
                    text.push_str(&format!("- {reference}\n"));
                }
                text.push('\n');
            }
        }
        text.push_str(
            "This guidance is derived from the rule documentation and the finding \
evidence. It is not an independent security judgment and does not change the \
deterministic result.",
        );

        Explanation {
            text,
            ai_generated: false,
            label: OFFLINE_LABEL,
        }
    }

    fn summarize(&self, findings: &[Finding]) -> Explanation {
        let mut counts = [0usize; 5];
        let mut categories: Vec<Category> = Vec::new();
        for finding in findings {
            counts[finding.severity.rank() as usize] += 1;
            if !categories.contains(&finding.category) {
                categories.push(finding.category);
            }
        }
        categories.sort();

        let mut text = format!(
            "Scanned with {} deterministic findings: critical {}, high {}, medium {}, low {}, info {}.\n\n",
            findings.len(),
            counts[4],
            counts[3],
            counts[2],
            counts[1],
            counts[0]
        );
        if !categories.is_empty() {
            text.push_str("Categories observed: ");
            text.push_str(
                &categories
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            text.push_str(".\n\n");
        }
        text.push_str(
            "Prioritize critical and high findings first, and treat low-confidence \
findings as review prompts rather than confirmed vulnerabilities.",
        );

        Explanation {
            text,
            ai_generated: false,
            label: OFFLINE_LABEL,
        }
    }
}

/// Builds the configured assistant.
///
/// Returns offline [`RuleGuidance`] when assistance is disabled. When assistance
/// is enabled, it fails closed unless a bundled provider is available (none is,
/// by design, until a provider is implemented with the required safeguards).
pub fn assistant(config: &AiConfig) -> Result<RuleGuidance, AiError> {
    if !config.enabled {
        return Ok(RuleGuidance);
    }
    match &config.provider {
        None => Err(AiError::NotConfigured),
        Some(provider) => Err(AiError::ProviderUnavailable(provider.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Category;
    use crate::confidence::Confidence;
    use crate::finding::SourceLocation;
    use crate::rule::RuleMetadata;
    use crate::severity::Severity;

    fn meta() -> RuleMetadata {
        RuleMetadata::new(
            "SS-001",
            "Title",
            "description of the problem with enough characters",
            Category::Authorization,
            Severity::High,
            Confidence::Medium,
            "call require_auth",
        )
        .with_references(&["https://example.com"])
    }

    fn finding() -> Finding {
        Finding::new(
            &meta(),
            SourceLocation::new("src/lib.rs", 5),
            "evidence here",
        )
    }

    #[test]
    fn disabled_by_default_returns_offline_guidance() {
        let config = AiConfig::default();
        assert!(!config.enabled);
        assert!(assistant(&config).is_ok());
    }

    #[test]
    fn enabling_without_provider_fails_closed() {
        let config = AiConfig {
            enabled: true,
            provider: None,
            model: None,
        };
        assert_eq!(assistant(&config).unwrap_err(), AiError::NotConfigured);
    }

    #[test]
    fn enabling_with_unknown_provider_fails_closed() {
        let config = AiConfig {
            enabled: true,
            provider: Some("no-such-provider".into()),
            model: None,
        };
        assert!(matches!(
            assistant(&config),
            Err(AiError::ProviderUnavailable(_))
        ));
    }

    #[test]
    fn guidance_is_labeled_and_uses_existing_evidence() {
        let explanation = RuleGuidance.explain_finding(&finding(), Some(&meta()));
        assert!(!explanation.ai_generated);
        assert_eq!(explanation.label, OFFLINE_LABEL);
        assert!(explanation.text.contains("evidence here"));
        assert!(explanation.text.contains("src/lib.rs:5"));
        assert!(explanation.text.contains("call require_auth"));
    }

    #[test]
    fn summary_reports_existing_findings_only() {
        let findings = vec![finding()];
        let explanation = RuleGuidance.summarize(&findings);
        assert!(explanation.text.contains("1 deterministic findings"));
        assert!(explanation.text.contains("critical 0, high 1"));
    }
}
