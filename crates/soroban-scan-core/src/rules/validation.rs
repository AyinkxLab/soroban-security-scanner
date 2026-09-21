//! Rule metadata and documentation validation.
//!
//! These checks keep the rule catalog consistent and contributor-friendly. They
//! are exercised both by unit tests and by a repository-level integration test.

use std::collections::BTreeSet;
use std::path::Path;

use crate::confidence::Confidence;
use crate::registry::{is_valid_rule_id, RuleRegistry};
use crate::rule::RuleMetadata;
use crate::severity::Severity;

/// Minimum length for a rule description.
pub const MIN_DESCRIPTION_LEN: usize = 40;

/// Validates a single rule's metadata, returning human-readable issues.
pub fn validate_metadata(meta: &RuleMetadata) -> Vec<String> {
    let mut issues = Vec::new();

    if !is_valid_rule_id(meta.id) {
        issues.push(format!("invalid rule id: {}", meta.id));
    }
    if meta.title.trim().is_empty() {
        issues.push(format!("{}: title is empty", meta.id));
    }
    if meta.description.trim().len() < MIN_DESCRIPTION_LEN {
        issues.push(format!(
            "{}: description is too short (min {MIN_DESCRIPTION_LEN} chars)",
            meta.id
        ));
    }
    if meta.remediation.trim().is_empty() {
        issues.push(format!("{}: remediation is empty", meta.id));
    }
    if meta.references.is_empty() {
        issues.push(format!("{}: at least one reference is required", meta.id));
    }
    for reference in meta.references {
        if !(reference.starts_with("https://") || reference.starts_with("http://")) {
            issues.push(format!("{}: reference is not a URL: {reference}", meta.id));
        }
    }

    // A heuristic cannot honestly claim both maximum impact and maximum
    // certainty. Reject the most misleading combination.
    if meta.default_severity == Severity::Critical && meta.default_confidence == Confidence::High {
        issues.push(format!(
            "{}: critical severity with high confidence is not allowed for heuristics",
            meta.id
        ));
    }

    issues
}

/// Validates every rule in a registry.
pub fn validate_registry(registry: &RuleRegistry) -> Vec<String> {
    let mut issues = Vec::new();
    let mut seen = BTreeSet::new();
    for meta in registry.metadata_sorted() {
        if !seen.insert(meta.id) {
            issues.push(format!("duplicate rule id: {}", meta.id));
        }
        issues.extend(validate_metadata(meta));
    }
    issues
}

/// Validates that every rule has a documentation file at
/// `<docs_root>/rules/detectors/<ID>.md`.
pub fn validate_docs(registry: &RuleRegistry, docs_root: &Path) -> Vec<String> {
    let dir = docs_root.join("rules").join("detectors");
    let mut issues = Vec::new();
    for meta in registry.metadata_sorted() {
        let path = dir.join(format!("{}.md", meta.id));
        if !path.exists() {
            issues.push(format!(
                "{}: missing documentation at {}",
                meta.id,
                path.display()
            ));
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Category;

    fn good_meta() -> RuleMetadata {
        RuleMetadata::new(
            "SS-100",
            "Example",
            "This description is comfortably longer than the minimum length.",
            Category::Authorization,
            Severity::High,
            Confidence::Medium,
            "Do the secure thing.",
        )
        .with_references(&["https://example.com/doc"])
    }

    #[test]
    fn accepts_valid_metadata() {
        assert!(validate_metadata(&good_meta()).is_empty());
    }

    #[test]
    fn rejects_short_description_and_missing_references() {
        let meta = RuleMetadata::new(
            "SS-101",
            "T",
            "too short",
            Category::Storage,
            Severity::Low,
            Confidence::Low,
            "",
        );
        let issues = validate_metadata(&meta);
        assert!(issues.iter().any(|i| i.contains("description")));
        assert!(issues.iter().any(|i| i.contains("remediation")));
        assert!(issues.iter().any(|i| i.contains("reference")));
    }

    #[test]
    fn rejects_bad_reference_url() {
        let mut meta = good_meta();
        meta.references = &["not-a-url"];
        assert!(validate_metadata(&meta)
            .iter()
            .any(|i| i.contains("not a URL")));
    }

    #[test]
    fn rejects_critical_high_confidence() {
        let mut meta = good_meta();
        meta.default_severity = Severity::Critical;
        meta.default_confidence = Confidence::High;
        assert!(validate_metadata(&meta)
            .iter()
            .any(|i| i.contains("critical severity with high confidence")));
    }

    #[test]
    fn built_in_registry_is_valid() {
        let registry = RuleRegistry::with_default_rules();
        assert!(validate_registry(&registry).is_empty());
    }
}
