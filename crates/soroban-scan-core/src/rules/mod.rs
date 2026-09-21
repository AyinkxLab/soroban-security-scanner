//! Built-in security rules (detectors).
//!
//! Each detector documents the problem it addresses, the evidence it relies on,
//! its limitations, and its known false positives. Detectors must be
//! deterministic and must only report findings backed by concrete evidence.
//!
//! Detector documentation lives in `docs/rules/detectors/`.

pub mod util;
pub mod validation;

mod cross_contract_auth;
mod hardcoded_address;
mod missing_authorization;
mod panic_prone;
mod ttl_management;
mod unbounded_iteration;
mod unsafe_code;

#[cfg(test)]
pub mod test_support;

use crate::registry::{RegistryError, RuleRegistry};

/// Registers every built-in rule with the registry.
pub fn register_all(registry: &mut RuleRegistry) -> Result<(), RegistryError> {
    registry.register(Box::new(missing_authorization::MissingAuthorization))?;
    registry.register(Box::new(cross_contract_auth::CrossContractAuth))?;
    registry.register(Box::new(unbounded_iteration::UnboundedIteration))?;
    registry.register(Box::new(panic_prone::PanicProne))?;
    registry.register(Box::new(unsafe_code::UnsafeCode))?;
    registry.register(Box::new(ttl_management::TtlManagement))?;
    registry.register(Box::new(hardcoded_address::HardcodedAddress))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::registry::RuleRegistry;

    #[test]
    fn default_registry_contains_all_detectors() {
        let registry = RuleRegistry::with_default_rules();
        let ids: Vec<_> = registry.metadata_sorted().iter().map(|m| m.id).collect();
        assert_eq!(
            ids,
            vec!["SS-001", "SS-002", "SS-003", "SS-004", "SS-005", "SS-006", "SS-007"]
        );
    }
}
