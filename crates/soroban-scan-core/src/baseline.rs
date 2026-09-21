//! Baselines: stable finding fingerprints for tracking new vs. known issues.
//!
//! A baseline records the fingerprints of accepted findings. Comparing a scan
//! against a baseline identifies findings that are new, already known, or
//! resolved. Fingerprints deliberately exclude line numbers so unrelated code
//! shifts do not resurrect known findings.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::error::ScanError;
use crate::finding::Finding;
use crate::severity::Severity;

/// Current baseline file format version.
pub const BASELINE_VERSION: u32 = 1;

/// A single baseline entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineEntry {
    /// Stable finding fingerprint.
    pub fingerprint: String,
    /// Rule id.
    pub rule_id: String,
    /// Scan-root-relative file.
    pub file: String,
    /// Severity at the time the baseline was written.
    pub severity: Severity,
    /// Finding title.
    pub title: String,
}

/// A baseline document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    /// Format version.
    pub version: u32,
    /// Tool that produced the baseline.
    pub tool: String,
    /// Tool version that produced the baseline.
    pub tool_version: String,
    /// Accepted findings, sorted by fingerprint for determinism.
    pub findings: Vec<BaselineEntry>,
}

impl Baseline {
    /// Builds a baseline from current findings.
    pub fn from_findings(findings: &[Finding]) -> Self {
        let mut entries: Vec<BaselineEntry> = findings
            .iter()
            .map(|f| BaselineEntry {
                fingerprint: f.fingerprint.clone(),
                rule_id: f.rule_id.clone(),
                file: f.location.file.to_string_lossy().replace('\\', "/"),
                severity: f.severity,
                title: f.title.clone(),
            })
            .collect();
        entries.sort_by(|a, b| a.fingerprint.cmp(&b.fingerprint));
        entries.dedup_by(|a, b| a.fingerprint == b.fingerprint);
        Baseline {
            version: BASELINE_VERSION,
            tool: crate::TOOL_NAME.to_string(),
            tool_version: crate::VERSION.to_string(),
            findings: entries,
        }
    }

    /// Fingerprint set for fast membership tests.
    pub fn fingerprints(&self) -> BTreeSet<&str> {
        self.findings
            .iter()
            .map(|entry| entry.fingerprint.as_str())
            .collect()
    }

    /// Returns true if a finding is present in this baseline.
    pub fn contains(&self, finding: &Finding) -> bool {
        self.fingerprints().contains(finding.fingerprint.as_str())
    }

    /// Serializes the baseline to pretty JSON.
    pub fn to_json(&self) -> Result<String, ScanError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ScanError::Baseline(format!("cannot serialize baseline: {e}")))
    }

    /// Parses a baseline from JSON.
    pub fn from_json(text: &str) -> Result<Self, ScanError> {
        let baseline: Baseline = serde_json::from_str(text)
            .map_err(|e| ScanError::Baseline(format!("invalid baseline JSON: {e}")))?;
        if baseline.version != BASELINE_VERSION {
            return Err(ScanError::Baseline(format!(
                "unsupported baseline version {} (expected {})",
                baseline.version, BASELINE_VERSION
            )));
        }
        Ok(baseline)
    }
}

/// Result of comparing findings against a baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// Indices (into the findings slice) of findings not in the baseline.
    pub new: Vec<usize>,
    /// Indices of findings already present in the baseline.
    pub existing: Vec<usize>,
    /// Baseline entries no longer present in the scan.
    pub resolved: Vec<BaselineEntry>,
}

impl Comparison {
    /// Number of new findings.
    pub fn new_count(&self) -> usize {
        self.new.len()
    }

    /// Number of existing (baselined) findings.
    pub fn existing_count(&self) -> usize {
        self.existing.len()
    }

    /// Number of resolved findings.
    pub fn resolved_count(&self) -> usize {
        self.resolved.len()
    }
}

/// Compares findings against a baseline by fingerprint.
pub fn compare(findings: &[Finding], baseline: &Baseline) -> Comparison {
    let baseline_fingerprints = baseline.fingerprints();
    let mut current: BTreeSet<&str> = BTreeSet::new();

    let mut new = Vec::new();
    let mut existing = Vec::new();
    for (index, finding) in findings.iter().enumerate() {
        current.insert(finding.fingerprint.as_str());
        if baseline_fingerprints.contains(finding.fingerprint.as_str()) {
            existing.push(index);
        } else {
            new.push(index);
        }
    }

    let resolved = baseline
        .findings
        .iter()
        .filter(|entry| !current.contains(entry.fingerprint.as_str()))
        .cloned()
        .collect();

    Comparison {
        new,
        existing,
        resolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Category;
    use crate::confidence::Confidence;
    use crate::finding::{Finding, SourceLocation};
    use crate::rule::RuleMetadata;

    fn meta() -> RuleMetadata {
        RuleMetadata::new(
            "SS-001",
            "Test",
            "test description long enough for validation purposes",
            Category::Authorization,
            Severity::High,
            Confidence::Medium,
            "fix",
        )
    }

    fn finding(file: &str, line: usize, evidence: &str) -> Finding {
        Finding::new(&meta(), SourceLocation::new(file, line), evidence)
    }

    #[test]
    fn baseline_round_trips_through_json() {
        let findings = vec![
            finding("src/lib.rs", 10, "a"),
            finding("src/lib.rs", 20, "b"),
        ];
        let baseline = Baseline::from_findings(&findings);
        let json = baseline.to_json().unwrap();
        let parsed = Baseline::from_json(&json).unwrap();
        assert_eq!(baseline, parsed);
        assert_eq!(parsed.findings.len(), 2);
    }

    #[test]
    fn rejects_wrong_version() {
        let text = r#"{"version":99,"tool":"soroban-scan","tool_version":"0.1.0","findings":[]}"#;
        assert!(matches!(
            Baseline::from_json(text),
            Err(ScanError::Baseline(_))
        ));
    }

    #[test]
    fn comparison_identifies_new_and_resolved() {
        let old = finding("src/lib.rs", 10, "keep");
        let baseline = Baseline::from_findings(std::slice::from_ref(&old));

        let kept = finding("src/lib.rs", 999, "keep"); // line shift does not matter
        let fresh = finding("src/lib.rs", 20, "brand new");
        let findings = vec![kept, fresh];

        let comparison = compare(&findings, &baseline);
        assert_eq!(comparison.new_count(), 1);
        assert_eq!(comparison.existing_count(), 1);
        assert_eq!(comparison.resolved_count(), 0);
    }

    #[test]
    fn resolved_findings_are_reported() {
        let baseline = Baseline::from_findings(&[finding("src/lib.rs", 10, "gone")]);
        let comparison = compare(&[], &baseline);
        assert_eq!(comparison.resolved_count(), 1);
        assert_eq!(comparison.new_count(), 0);
    }

    #[test]
    fn baseline_output_is_deterministic() {
        let findings = vec![finding("b.rs", 1, "x"), finding("a.rs", 2, "y")];
        let first = Baseline::from_findings(&findings).to_json().unwrap();
        let second = Baseline::from_findings(&findings).to_json().unwrap();
        assert_eq!(first, second);
    }
}
