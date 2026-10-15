use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use crate::types::{
    CallPermission,
    GridAccount::{SpendInfo, SpendPeriod},
};

/// Represents key permissions.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
#[cfg_attr(test, strum_discriminants(derive(strum::EnumIter), doc = "Key permissions"))]
#[serde(tag = "type")]
pub enum Permission {
    /// Call permission.
    #[serde(rename = "call")]
    Call(CallPermission),
    /// Spend permission.
    #[serde(rename = "spend")]
    Spend(SpendPermission),
    /// Signature verification permission.
    #[serde(rename = "signatureVerification")]
    SignatureVerification(SignatureVerificationPermission),
}

impl From<CallPermission> for Permission {
    fn from(perm: CallPermission) -> Self {
        Permission::Call(perm)
    }
}

impl From<SpendPermission> for Permission {
    fn from(perm: SpendPermission) -> Self {
        Permission::Spend(perm)
    }
}

impl From<SignatureVerificationPermission> for Permission {
    fn from(perm: SignatureVerificationPermission) -> Self {
        Permission::SignatureVerification(perm)
    }
}

/// Represents a signature verification permission.
///
/// Approves a set of contracts as ERC-1271 signature checkers for a key, i.e. the account will
/// return the ERC-1271 magic value for signatures produced by the key only when `isValidSignature`
/// is called by one of these addresses.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignatureVerificationPermission {
    /// The addresses of the approved signature checkers.
    pub addresses: Vec<Address>,
}

/// Represents spend permissions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpendPermission {
    /// The spending limit.
    pub limit: U256,
    /// The spending period.
    pub period: SpendPeriod,
    /// The token address.
    #[serde(default)]
    pub token: Address,
}

impl From<SpendInfo> for SpendPermission {
    fn from(permission: SpendInfo) -> Self {
        Self { limit: permission.limit, period: permission.period, token: permission.token }
    }
}

#[cfg(test)]
mod tests {
    use super::{Permission, SignatureVerificationPermission};
    use crate::types::CallPermission;
    use alloy::primitives::{Address, address, fixed_bytes};

    #[test]
    fn signature_verification_permission_roundtrip() {
        let json = r#"{"type":"signatureVerification","addresses":["0x000000000000000000000000000000000000c0de"]}"#;
        let permission = serde_json::from_str::<Permission>(json).unwrap();
        assert_eq!(
            permission,
            Permission::SignatureVerification(SignatureVerificationPermission {
                addresses: vec![address!("0x000000000000000000000000000000000000c0de")],
            })
        );
        assert_eq!(serde_json::to_string(&permission).unwrap(), json);
    }

    #[test]
    fn deserialize_call_permission() {
        assert_eq!(
            serde_json::from_str::<CallPermission>(
                r#"{
                "to":"0x0000000000000000000000000000000000000000",
                "selector":"transfer(address,uint256)"
            }"#
            )
            .unwrap(),
            CallPermission { to: Address::ZERO, selector: fixed_bytes!("0xa9059cbb") }
        );

        assert_eq!(
            serde_json::from_str::<CallPermission>(
                r#"
                {
                "to":"0x0000000000000000000000000000000000000000",
                "selector":"0xa9059cbb"
            }"#
            )
            .unwrap(),
            CallPermission { to: Address::ZERO, selector: fixed_bytes!("0xa9059cbb") }
        )
    }
}
