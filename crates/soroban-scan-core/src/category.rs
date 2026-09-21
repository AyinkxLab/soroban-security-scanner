//! Finding categories.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Security category a rule belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// Caller authorization (`require_auth`).
    Authorization,
    /// Authentication of identities.
    Authentication,
    /// Access control and privileged operations.
    AccessControl,
    /// Cross-contract calls.
    CrossContract,
    /// Token handling and transfers.
    Token,
    /// Persistent/temporary/instance storage usage.
    Storage,
    /// Arithmetic and numeric safety.
    Arithmetic,
    /// Error handling and panics.
    ErrorHandling,
    /// Event emission.
    Events,
    /// Resource usage, TTL, and fees.
    ResourceUsage,
    /// Contract configuration.
    Configuration,
    /// Dependency and supply-chain risk.
    Dependencies,
    /// Unsafe Rust constructs.
    Unsafe,
}

impl Category {
    /// Stable kebab-case name.
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Authorization => "authorization",
            Category::Authentication => "authentication",
            Category::AccessControl => "access-control",
            Category::CrossContract => "cross-contract",
            Category::Token => "token",
            Category::Storage => "storage",
            Category::Arithmetic => "arithmetic",
            Category::ErrorHandling => "error-handling",
            Category::Events => "events",
            Category::ResourceUsage => "resource-usage",
            Category::Configuration => "configuration",
            Category::Dependencies => "dependencies",
            Category::Unsafe => "unsafe",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "authorization" => Ok(Category::Authorization),
            "authentication" => Ok(Category::Authentication),
            "access-control" => Ok(Category::AccessControl),
            "cross-contract" => Ok(Category::CrossContract),
            "token" => Ok(Category::Token),
            "storage" => Ok(Category::Storage),
            "arithmetic" => Ok(Category::Arithmetic),
            "error-handling" => Ok(Category::ErrorHandling),
            "events" => Ok(Category::Events),
            "resource-usage" => Ok(Category::ResourceUsage),
            "configuration" => Ok(Category::Configuration),
            "dependencies" => Ok(Category::Dependencies),
            "unsafe" => Ok(Category::Unsafe),
            other => Err(format!("unknown category: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kebab_and_snake() {
        assert_eq!(
            "access-control".parse::<Category>().unwrap(),
            Category::AccessControl
        );
        assert_eq!(
            "access_control".parse::<Category>().unwrap(),
            Category::AccessControl
        );
    }

    #[test]
    fn round_trips() {
        for c in [
            Category::Authorization,
            Category::CrossContract,
            Category::ResourceUsage,
        ] {
            assert_eq!(c.as_str().parse::<Category>().unwrap(), c);
        }
    }
}
