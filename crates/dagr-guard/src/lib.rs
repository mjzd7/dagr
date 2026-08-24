pub mod alias;
pub mod capabilities;
pub mod checker;
pub mod ci;
pub mod infer;
pub mod licenses;
pub mod rules;
pub mod sanitizer;
pub mod secrets;

pub use alias::AliasMap;
pub use capabilities::{CapabilityGrant, CredentialBroker, Permission};
pub use checker::{checker_relative_candidates, ArchitectureGuard, Violation};
pub use ci::CiGuardReport;
pub use infer::ArchitectureInferrer;
pub use licenses::{
    check_declared_licenses, check_dependency_licenses, DepLicenseViolation, LicenseViolation,
    DEFAULT_ALLOWLIST, DEFAULT_RUNTIME_ALLOWLIST_EXTRAS,
};
pub use rules::{BoundaryRule, LimitsConfig, RuleConfig, SecurityConfig};
pub use sanitizer::{MutationRiskLevel, ProgressivePermissionGate, ZeroTrustSanitizer};
pub use secrets::{is_likely_generated, SecretFinding, SecretScanner, SuppressionBaseline};
