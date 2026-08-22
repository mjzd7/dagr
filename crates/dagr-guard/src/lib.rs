pub mod alias;
pub mod capabilities;
pub mod checker;
pub mod ci;
pub mod infer;
pub mod rules;
pub mod sanitizer;

pub use alias::AliasMap;
pub use capabilities::{CapabilityGrant, CredentialBroker, Permission};
pub use checker::{ArchitectureGuard, Violation, checker_relative_candidates};
pub use ci::CiGuardReport;
pub use infer::ArchitectureInferrer;
pub use rules::{BoundaryRule, LimitsConfig, RuleConfig, SecurityConfig};
pub use sanitizer::{MutationRiskLevel, ProgressivePermissionGate, ZeroTrustSanitizer};
