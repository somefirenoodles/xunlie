//! Pure, deterministic domain types for Xunlie.
//!
//! This crate deliberately has no filesystem, network, clock, process, or
//! randomness APIs. Callers supply every value that can affect a verdict.

#![forbid(unsafe_code)]

mod canonical;
mod diagnostic;
mod digest;
mod ir;
mod resolution;

pub use canonical::CanonicalizationError;
pub use diagnostic::{Diagnostic, DiagnosticRelated, Severity, SourceLocation};
pub use digest::{DigestParseError, Sha256Digest};
pub use ir::{
    CONTRACT_SCHEMA_VERSION, ContractIr, ContractMetadata, Producer, Requirement, RequirementId,
    RequirementIdError, RequirementKind, RequirementPriority, SourceIdentity, SourceIdentityError,
    SourceRecord,
};
pub use resolution::{
    Conflict, ConflictKind, History, HistoryEvent, Operation, Precedence, ResolutionAction,
    ResolutionDecision, ResolutionFailure, ResolutionPolicy, ResolvedContract, resolve_history,
};
