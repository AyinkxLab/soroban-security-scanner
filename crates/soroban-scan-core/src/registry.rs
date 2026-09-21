//! Rule registry.

use crate::rule::{Rule, RuleMetadata};

/// Errors raised when registering rules.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RegistryError {
    /// A rule with the same id is already registered.
    #[error("duplicate rule id: {0}")]
    Duplicate(String),
    /// A rule id is malformed.
    #[error("invalid rule id: {0} (expected SS-###)")]
    InvalidId(String),
}

/// Returns true if `id` matches the `SS-###` convention.
pub fn is_valid_rule_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("SS-") else {
        return false;
    };
    rest.len() == 3 && rest.chars().all(|c| c.is_ascii_digit())
}

/// A registry of security rules.
#[derive(Default)]
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        RuleRegistry { rules: Vec::new() }
    }

    /// Creates a registry with all built-in rules registered.
    pub fn with_default_rules() -> Self {
        let mut registry = RuleRegistry::new();
        registry
            .register_defaults()
            .expect("built-in rule ids must be valid and unique");
        registry
    }

    /// Registers a rule, rejecting duplicate or malformed ids.
    pub fn register(&mut self, rule: Box<dyn Rule>) -> Result<(), RegistryError> {
        let id = rule.metadata().id;
        if !is_valid_rule_id(id) {
            return Err(RegistryError::InvalidId(id.to_string()));
        }
        if self.rules.iter().any(|r| r.metadata().id == id) {
            return Err(RegistryError::Duplicate(id.to_string()));
        }
        self.rules.push(rule);
        Ok(())
    }

    /// Registers built-in rules. Populated as detectors are implemented.
    fn register_defaults(&mut self) -> Result<(), RegistryError> {
        crate::rules::register_all(self)
    }

    /// Number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns true if no rules are registered.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// All registered rules, in registration order.
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Finds a rule by id.
    pub fn by_id(&self, id: &str) -> Option<&dyn Rule> {
        self.rules
            .iter()
            .find(|r| r.metadata().id == id)
            .map(|r| r.as_ref())
    }

    /// Metadata for all rules, sorted by id.
    pub fn metadata_sorted(&self) -> Vec<&RuleMetadata> {
        let mut metas: Vec<&RuleMetadata> = self.rules.iter().map(|r| r.metadata()).collect();
        metas.sort_by_key(|m| m.id);
        metas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Category;
    use crate::confidence::Confidence;
    use crate::context::AnalysisContext;
    use crate::finding::Finding;
    use crate::severity::Severity;

    struct Dummy {
        meta: RuleMetadata,
    }

    impl Dummy {
        fn new(id: &'static str) -> Self {
            Dummy {
                meta: RuleMetadata::new(
                    id,
                    "Dummy",
                    "dummy rule",
                    Category::Authorization,
                    Severity::Low,
                    Confidence::Low,
                    "n/a",
                ),
            }
        }
    }

    impl Rule for Dummy {
        fn metadata(&self) -> &RuleMetadata {
            &self.meta
        }
        fn analyze(&self, _ctx: &AnalysisContext<'_>, _out: &mut Vec<Finding>) {}
    }

    #[test]
    fn rejects_duplicate_ids() {
        let mut reg = RuleRegistry::new();
        reg.register(Box::new(Dummy::new("SS-001"))).unwrap();
        assert_eq!(
            reg.register(Box::new(Dummy::new("SS-001"))),
            Err(RegistryError::Duplicate("SS-001".into()))
        );
    }

    #[test]
    fn rejects_malformed_ids() {
        let mut reg = RuleRegistry::new();
        assert!(matches!(
            reg.register(Box::new(Dummy::new("AUTH-1"))),
            Err(RegistryError::InvalidId(_))
        ));
    }

    #[test]
    fn validates_rule_id_shape() {
        assert!(is_valid_rule_id("SS-001"));
        assert!(!is_valid_rule_id("SS-1"));
        assert!(!is_valid_rule_id("ss-001"));
        assert!(!is_valid_rule_id("SS-00a"));
    }

    #[test]
    fn metadata_is_sorted() {
        let mut reg = RuleRegistry::new();
        reg.register(Box::new(Dummy::new("SS-002"))).unwrap();
        reg.register(Box::new(Dummy::new("SS-001"))).unwrap();
        let ids: Vec<_> = reg.metadata_sorted().iter().map(|m| m.id).collect();
        assert_eq!(ids, vec!["SS-001", "SS-002"]);
    }
}
