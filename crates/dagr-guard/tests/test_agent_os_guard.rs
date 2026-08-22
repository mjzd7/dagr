use dagr_core::Result;
use dagr_guard::{CapabilityGrant, CredentialBroker, Permission};
use uuid::Uuid;

#[test]
fn test_capability_tokens_and_credential_broker() -> Result<()> {
    let run_id = Uuid::new_v4();
    let secret_key = b"super-secret-hmac-master-key-123456";

    // 1. Valid Grant
    let grant = CapabilityGrant::new_signed(
        run_id,
        "tenant-gamma",
        vec![Permission::ReadAST, Permission::ExecuteSubprocess],
        3600, // 1 hour TTL
        secret_key,
    );

    assert!(grant.verify(secret_key).is_ok());
    assert!(grant.has_permission(&Permission::ReadAST));
    assert!(grant.has_permission(&Permission::ExecuteSubprocess));
    assert!(!grant.has_permission(&Permission::MutateCoWShadow));

    // 2. Tampered Secret Verification
    let fake_key = b"wrong-key-attempting-forgery-999999";
    assert!(
        grant.verify(fake_key).is_err(),
        "Tampered secret must fail verification"
    );

    // 3. Credential Broker Isolation
    let broker = CredentialBroker::new();
    broker.register_secret(
        "ref://github_pat",
        "ghp_RealSecretTokenValueThatNeverEntersPrompts",
    );

    let resolved = broker.resolve_handle(&grant, "ref://github_pat", secret_key)?;
    assert_eq!(resolved, "ghp_RealSecretTokenValueThatNeverEntersPrompts");

    // 4. Grant lacking ExecuteSubprocess permission
    let restricted_grant = CapabilityGrant::new_signed(
        run_id,
        "tenant-gamma",
        vec![Permission::ReadAST],
        3600,
        secret_key,
    );
    assert!(broker
        .resolve_handle(&restricted_grant, "ref://github_pat", secret_key)
        .is_err());

    Ok(())
}
