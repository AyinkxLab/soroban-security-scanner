//! Scan and rule configuration.
//!
//! Repository-provided configuration is treated as untrusted data. No option
//! here can enable code execution or network access.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::confidence::Confidence;
use crate::severity::Severity;

/// Rule selection and overrides.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RulesConfig {
    /// Rule ids to disable.
    pub disabled: Vec<String>,
    /// If non-empty, only these rule ids are enabled.
    pub enabled: Vec<String>,
    /// Per-rule severity overrides.
    pub severity_overrides: BTreeMap<String, Severity>,
    /// Per-rule confidence overrides.
    pub confidence_overrides: BTreeMap<String, Confidence>,
}

impl RulesConfig {
    /// Returns true if the rule id is enabled under this configuration.
    pub fn is_enabled(&self, id: &str) -> bool {
        if self.disabled.iter().any(|r| r == id) {
            return false;
        }
        if !self.enabled.is_empty() {
            return self.enabled.iter().any(|r| r == id);
        }
        true
    }
}

/// Complete scan configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScanConfig {
    /// Minimum severity to report.
    pub min_severity: Severity,
    /// Minimum confidence to report.
    pub min_confidence: Confidence,
    /// Glob patterns (relative to the scan root) to exclude from discovery.
    pub exclude: Vec<String>,
    /// Whether to follow symbolic links during discovery.
    pub follow_symlinks: bool,
    /// Rule selection and overrides.
    pub rules: RulesConfig,
}

impl Default for ScanConfig {
    fn default() -> Self {
        ScanConfig {
            min_severity: Severity::Info,
            min_confidence: Confidence::Low,
            exclude: Vec::new(),
            follow_symlinks: false,
            rules: RulesConfig::default(),
        }
    }
}

impl ScanConfig {
    /// Builds [`crate::discovery::DiscoveryOptions`] from this configuration.
    pub fn discovery_options(&self) -> crate::discovery::DiscoveryOptions {
        crate::discovery::DiscoveryOptions {
            follow_symlinks: self.follow_symlinks,
            exclude: self.exclude.clone(),
        }
    }

    /// Parses a configuration from TOML text.
    pub fn from_toml(text: &str) -> Result<Self, crate::error::ScanError> {
        text.parse::<toml::Value>()
            .map_err(|e| crate::error::ScanError::Config(format!("invalid config TOML: {e}")))
            .and_then(|value| {
                value
                    .try_into()
                    .map_err(|e| crate::error::ScanError::Config(format!("invalid config: {e}")))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_permissive_and_safe() {
        let config = ScanConfig::default();
        assert_eq!(config.min_severity, Severity::Info);
        assert_eq!(config.min_confidence, Confidence::Low);
        assert!(!config.follow_symlinks);
    }

    #[test]
    fn rule_enable_disable_logic() {
        let rules = RulesConfig::default();
        assert!(rules.is_enabled("SS-001"));

        let rules = RulesConfig {
            disabled: vec!["SS-001".into()],
            ..RulesConfig::default()
        };
        assert!(!rules.is_enabled("SS-001"));

        let only = RulesConfig {
            enabled: vec!["SS-002".into()],
            ..RulesConfig::default()
        };
        assert!(only.is_enabled("SS-002"));
        assert!(!only.is_enabled("SS-001"));
    }

    #[test]
    fn parses_toml_config() {
        let config = ScanConfig::from_toml(
            r#"
min_severity = "medium"
min_confidence = "low"
exclude = ["vendor/**"]

[rules]
disabled = ["SS-010"]
"#,
        )
        .unwrap();
        assert_eq!(config.min_severity, Severity::Medium);
        assert!(!config.rules.is_enabled("SS-010"));
    }

    #[test]
    fn rejects_unknown_fields() {
        let err = ScanConfig::from_toml("mystery = true").unwrap_err();
        assert!(matches!(err, crate::error::ScanError::Config(_)));
    }
}
