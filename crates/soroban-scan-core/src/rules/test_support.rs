//! Test-only helpers for exercising rules without touching the filesystem.

use crate::confidence::Confidence;
use crate::config::ScanConfig;
use crate::engine;
use crate::project::LoadedProject;
use crate::registry::RuleRegistry;
use crate::rule::Rule;
use crate::severity::Severity;

/// A flat summary of a finding, convenient for assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingSummary {
    /// Rule id.
    pub rule_id: String,
    /// Title.
    pub title: String,
    /// Severity.
    pub severity: Severity,
    /// Confidence.
    pub confidence: Confidence,
    /// Evidence text.
    pub evidence: String,
    /// 1-based line.
    pub line: usize,
}

/// Builds an in-memory loaded project from a single source string.
pub fn build_loaded(src: &str) -> LoadedProject {
    engine::synthetic_loaded(src, "src/lib.rs")
}

fn summarize(registry: &RuleRegistry, src: &str) -> Vec<FindingSummary> {
    let loaded = build_loaded(src);
    let outcome = engine::scan(&loaded, registry, &ScanConfig::default());
    outcome
        .findings
        .iter()
        .map(|f| FindingSummary {
            rule_id: f.rule_id.clone(),
            title: f.title.clone(),
            severity: f.severity,
            confidence: f.confidence,
            evidence: f.evidence.clone(),
            line: f.location.line,
        })
        .collect()
}

/// Scans a source string with a single rule.
pub fn scan_rule(src: &str, rule: Box<dyn Rule>) -> Vec<FindingSummary> {
    let mut registry = RuleRegistry::new();
    registry
        .register(rule)
        .expect("test rule must have a valid id");
    summarize(&registry, src)
}

/// Scans a source string with all built-in rules.
pub fn scan_all(src: &str) -> Vec<FindingSummary> {
    summarize(&RuleRegistry::with_default_rules(), src)
}
