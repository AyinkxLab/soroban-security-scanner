//! Classic Stellar address tooling.
//!
//! Demonstrates safe handling of classic Stellar strkeys using the official
//! `stellar-strkey` crate: validating account and contract addresses before they
//! are used to build transactions or construct Soroban clients. This is not a
//! Soroban contract; it is a host-side helper library.

use stellar_strkey::Strkey;

/// The kind of Stellar strkey an address encodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressKind {
    /// A classic Ed25519 account (`G...`).
    Account,
    /// A Soroban contract (`C...`).
    Contract,
}

/// Address validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    /// The string is not a valid strkey.
    Invalid,
    /// The strkey is valid but not the expected kind.
    WrongKind,
}

/// Classifies a Stellar address.
pub fn classify(address: &str) -> Result<AddressKind, AddressError> {
    match Strkey::from_string(address) {
        Ok(Strkey::PublicKeyEd25519(_)) => Ok(AddressKind::Account),
        Ok(Strkey::Contract(_)) => Ok(AddressKind::Contract),
        _ => Err(AddressError::Invalid),
    }
}

/// Returns an error unless the address is a valid account strkey.
pub fn require_account(address: &str) -> Result<(), AddressError> {
    match classify(address)? {
        AddressKind::Account => Ok(()),
        AddressKind::Contract => Err(AddressError::WrongKind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_addresses() {
        assert_eq!(classify("not-an-address"), Err(AddressError::Invalid));
    }
}
