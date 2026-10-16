use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use crate::types::{
    CallPermission,
    GridAccount::{SpendInfo, SpendPeriod},
};

/// Represents key permissions.
///
/// The set of variants is advertised to clients as [`PermissionType`]s in
/// `wallet_getCapabilities`, so adding a variant here automatically advertises it.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(
    name(PermissionType),
    derive(Serialize, Deserialize, Hash, PartialOrd, Ord, strum::EnumIter),
    serde(rename_all = "camelCase"),
    doc = "The kind of a key [`Permission`], as accepted in `authorizeKeys` and advertised in \
           `wallet_getCapabilities`."
)]
#[serde(tag = "type")]
pub enum Permission {
    /// Call permission.
    #[serde(rename = "call")]
    Call(CallPermission),
    /// Spend permission.
    #[serde(rename = "spend")]
    Spend(SpendPermission),
    /// Sign permission.
    #[serde(rename = "sign")]
    Sign(SignPermission),
}

impl PermissionType {
    /// Every permission type this relay accepts, in declaration order.
    pub fn all() -> Vec<Self> {
        <Self as strum::IntoEnumIterator>::iter().collect()
    }
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

impl From<SignPermission> for Permission {
    fn from(perm: SignPermission) -> Self {
        Permission::Sign(perm)
    }
}

/// Represents a sign permission: the key may sign for `verifier`.
///
/// Approves `verifier` as an ERC-1271 signature checker for the key, i.e. the account returns the
/// ERC-1271 magic value for signatures produced by the key only when `isValidSignature` is called
/// by `verifier`. One permission per verifier, mirroring `call` (one per target) and `spend` (one
/// per token). Maps to `setSignatureCheckerApproval(keyHash, verifier, true)`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignPermission {
    /// The contract that may accept the key's signatures.
    pub verifier: Address,
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
    use super::{Permission, PermissionType, SignPermission};
    use crate::types::CallPermission;
    use alloy::primitives::{Address, address, fixed_bytes};

    /// The advertised type names must be exactly the `type` tags clients send in
    /// `authorizeKeys`, otherwise feature detection lies.
    #[test]
    fn permission_type_names_match_permission_tags() {
        assert_eq!(
            serde_json::to_value(PermissionType::all()).unwrap(),
            serde_json::json!(["call", "spend", "sign"]),
        );
        for ty in PermissionType::all() {
            let tag = serde_json::to_value(ty).unwrap();
            let permission = match ty {
                PermissionType::Call => Permission::Call(CallPermission {
                    to: Address::ZERO,
                    selector: fixed_bytes!("0xa9059cbb"),
                }),
                PermissionType::Spend => Permission::Spend(super::SpendPermission {
                    limit: Default::default(),
                    period: crate::types::GridAccount::SpendPeriod::Day,
                    token: Address::ZERO,
                }),
                PermissionType::Sign => {
                    Permission::Sign(SignPermission { verifier: Address::ZERO })
                }
            };
            assert_eq!(serde_json::to_value(&permission).unwrap()["type"], tag);
            assert_eq!(PermissionType::from(&permission), ty);
        }
    }

    #[test]
    fn sign_permission_roundtrip() {
        let json = r#"{"type":"sign","verifier":"0x000000000000000000000000000000000000c0de"}"#;
        let permission = serde_json::from_str::<Permission>(json).unwrap();
        assert_eq!(
            permission,
            Permission::Sign(SignPermission {
                verifier: address!("0x000000000000000000000000000000000000c0de"),
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
