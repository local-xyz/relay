use super::{RelayStorage, StorageApi};
use alloy::primitives::Address;

async fn assert_email_attestation(storage: RelayStorage) {
    let account = Address::with_last_byte(1);
    let other = Address::with_last_byte(2);
    let empty = storage.get_onramp_contact_info(account).await.unwrap();
    assert_eq!(empty.email, None);
    assert_eq!(empty.email_verified_at, None);

    storage.add_unverified_email(account, "verified@example.com", "token").await.unwrap();
    let pending = storage.get_onramp_contact_info(account).await.unwrap();
    assert_eq!(pending.email.as_deref(), Some("verified@example.com"));
    assert_eq!(pending.email_verified_at, None);
    assert_eq!(storage.get_onramp_verification_status(account).await.unwrap().email, None);
    assert!(!storage.verify_email(account, "verified@example.com", "wrong").await.unwrap());
    assert_eq!(storage.get_onramp_contact_info(account).await.unwrap().email_verified_at, None);

    assert!(storage.verify_email(account, "verified@example.com", "token").await.unwrap());
    let verified = storage.get_onramp_contact_info(account).await.unwrap();
    let timestamp = verified.email_verified_at.expect("verified email must have evidence");
    assert!(timestamp > 0 && timestamp <= chrono::Utc::now().timestamp() as u64);

    assert_eq!(
        storage.get_onramp_verification_status(account).await.unwrap().email,
        Some(timestamp)
    );
    // A pending email on another account must never inherit verification.
    storage.add_unverified_email(other, "other@example.com", "other").await.unwrap();
    let selected = storage.get_onramp_contact_info(account).await.unwrap();
    assert_eq!(selected.email.as_deref(), Some("verified@example.com"));
    assert_eq!(selected.email_verified_at, Some(timestamp));
    assert_eq!(storage.get_onramp_contact_info(other).await.unwrap().email_verified_at, None);
}

#[tokio::test]
async fn memory_email_attestation() {
    assert_email_attestation(RelayStorage::in_memory()).await;
}

#[sqlx::test]
#[ignore = "requires PostgreSQL; run in the pg CI variant"]
async fn postgres_email_attestation(pool: sqlx::PgPool) {
    assert_email_attestation(RelayStorage::pg(pool.clone())).await;
    // PostgreSQL bigint casts round fractional seconds; Unix timestamps must truncate.
    sqlx::query("update emails set verified_at = timestamp '2026-01-01 00:00:00.9' where email = 'verified@example.com'")
        .execute(&pool).await.unwrap();
    let storage = RelayStorage::pg(pool);
    let account = Address::with_last_byte(1);
    assert_eq!(
        storage.get_onramp_contact_info(account).await.unwrap().email_verified_at,
        Some(1767225600)
    );
    assert_eq!(
        storage.get_onramp_verification_status(account).await.unwrap().email,
        Some(1767225600)
    );
}

#[test]
fn legacy_contact_response_has_no_email_attestation() {
    let response: crate::types::rpc::GetOnrampContactInfoResponse =
        serde_json::from_value(serde_json::json!({"email": "pending@example.com"})).unwrap();
    assert_eq!(response.email_verified_at, None);
    let response = crate::types::rpc::GetOnrampContactInfoResponse {
        email: Some("verified@example.com".into()),
        email_verified_at: Some(123),
        phone: None,
        phone_verified_at: None,
    };
    assert_eq!(serde_json::to_value(response).unwrap()["emailVerifiedAt"], 123);
}
