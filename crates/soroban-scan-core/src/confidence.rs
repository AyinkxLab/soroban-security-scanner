//! Finding confidence.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// How certain a detector is that a finding is real, based on the evidence it
/// can observe statically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Heuristic; may be a false positive; requires human review.
    Low,
    /// Strong evidence, but assumptions or context are required.
    Medium,
    /// Direct, unambiguous evidence in the source.
    High,
}

impl Confidence {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        }
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Confidence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "low" => Ok(Confidence::Low),
            "medium" | "med" => Ok(Confidence::Medium),
            "high" => Ok(Confidence::High),
            other => Err(format!("unknown confidence: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_ascending() {
        assert!(Confidence::High > Confidence::Medium);
        assert!(Confidence::Medium > Confidence::Low);
    }

    #[test]
    fn parses_names() {
        assert_eq!("low".parse::<Confidence>().unwrap(), Confidence::Low);
        assert!("nope".parse::<Confidence>().is_err());
    }
}
