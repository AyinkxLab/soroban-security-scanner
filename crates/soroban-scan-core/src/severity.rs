//! Finding severity.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Severity of a security finding, ordered from least to most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational; no direct security impact.
    Info,
    /// Minor weakness or hardening opportunity.
    Low,
    /// Security weakness that contributes to an exploit.
    Medium,
    /// Serious security impact under plausible conditions.
    High,
    /// Direct loss of funds or total loss of access control.
    Critical,
}

impl Severity {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    /// Numeric rank (higher is more severe).
    pub fn rank(self) -> u8 {
        match self {
            Severity::Info => 0,
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        }
    }

    /// Returns true if `self` is at least as severe as `threshold`.
    pub fn meets(self, threshold: Severity) -> bool {
        self.rank() >= threshold.rank()
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "info" => Ok(Severity::Info),
            "low" => Ok(Severity::Low),
            "medium" | "med" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" | "crit" => Ok(Severity::Critical),
            other => Err(format!("unknown severity: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_ascending() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }

    #[test]
    fn meets_threshold() {
        assert!(Severity::High.meets(Severity::Medium));
        assert!(Severity::Medium.meets(Severity::Medium));
        assert!(!Severity::Low.meets(Severity::Medium));
    }

    #[test]
    fn parses_names() {
        assert_eq!("HIGH".parse::<Severity>().unwrap(), Severity::High);
        assert_eq!("crit".parse::<Severity>().unwrap(), Severity::Critical);
        assert!("nope".parse::<Severity>().is_err());
    }
}
