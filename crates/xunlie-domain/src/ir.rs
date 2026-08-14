use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

use crate::{
    CanonicalizationError, Diagnostic, ResolutionPolicy, Sha256Digest, SourceLocation,
    canonical::to_canonical_json,
};

/// Schema identifier for the first persisted contract representation.
pub const CONTRACT_SCHEMA_VERSION: &str = "xunlie.contract/v1";

/// A non-empty, stable identity for a source (for example a URI or repository path).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceIdentity(String);

impl SourceIdentity {
    /// Validates and creates a source identity.
    pub fn new(value: impl Into<String>) -> Result<Self, SourceIdentityError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SourceIdentityError::Empty);
        }
        if value.chars().any(char::is_control) {
            return Err(SourceIdentityError::ControlCharacter);
        }
        Ok(Self(value))
    }

    /// Returns the identity as supplied by the caller.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for SourceIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SourceIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Why a source identity was rejected.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SourceIdentityError {
    /// Empty identities cannot be audited.
    #[error("source identity must not be empty")]
    Empty,
    /// Control characters make logs and serialized records ambiguous.
    #[error("source identity must not contain control characters")]
    ControlCharacter,
}

/// An identifier for a requirement in the effective contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RequirementId(String);

impl RequirementId {
    /// Creates an identifier containing ASCII letters, digits, `.`, `_`, `-`, `/`, or `:`.
    pub fn new(value: impl Into<String>) -> Result<Self, RequirementIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(RequirementIdError::Empty);
        }
        if value.len() > 128 {
            return Err(RequirementIdError::TooLong(value.len()));
        }
        if !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/' | b':')
        }) {
            return Err(RequirementIdError::InvalidCharacter);
        }
        Ok(Self(value))
    }

    /// Returns the normalized identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequirementId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for RequirementId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RequirementId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Why a requirement identifier was rejected.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum RequirementIdError {
    /// Empty identifiers are invalid.
    #[error("requirement id must not be empty")]
    Empty,
    /// IDs are bounded to keep reports and indexes predictable.
    #[error("requirement id contains {0} bytes; maximum is 128")]
    TooLong(usize),
    /// The identifier used a non-portable character.
    #[error("requirement id contains an invalid character")]
    InvalidCharacter,
}

/// Semantic category of a contract requirement.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RequirementKind {
    /// User-visible behavior.
    Functional,
    /// Quality or non-functional behavior.
    Quality,
    /// A hard restriction on possible implementations.
    Constraint,
    /// An acceptance oracle or expected observation.
    Acceptance,
    /// A condition that must remain true throughout a run.
    Invariant,
}

/// Contractual force of a requirement.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RequirementPriority {
    /// Required for the contract to pass.
    Must,
    /// Expected unless explicitly waived by later policy.
    Should,
    /// Optional behavior.
    May,
}

/// One normalized requirement in the effective contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Requirement {
    id: RequirementId,
    kind: RequirementKind,
    priority: RequirementPriority,
    statement: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    attributes: BTreeMap<String, String>,
}

impl Requirement {
    /// Creates a requirement and checks its local invariants.
    pub fn new(
        id: RequirementId,
        kind: RequirementKind,
        priority: RequirementPriority,
        statement: impl Into<String>,
    ) -> Result<Self, Box<Diagnostic>> {
        let requirement = Self {
            id,
            kind,
            priority,
            statement: statement.into(),
            attributes: BTreeMap::new(),
        };
        requirement.validate()?;
        Ok(requirement)
    }

    /// Returns the stable requirement identifier.
    #[must_use]
    pub const fn id(&self) -> &RequirementId {
        &self.id
    }

    /// Returns the normalized statement.
    #[must_use]
    pub fn statement(&self) -> &str {
        &self.statement
    }

    /// Returns the requirement kind.
    #[must_use]
    pub const fn kind(&self) -> RequirementKind {
        self.kind
    }

    /// Returns the requirement priority.
    #[must_use]
    pub const fn priority(&self) -> RequirementPriority {
        self.priority
    }

    /// Returns deterministic extension attributes.
    #[must_use]
    pub const fn attributes(&self) -> &BTreeMap<String, String> {
        &self.attributes
    }

    /// Validates invariants not expressible through deserialization types.
    pub fn validate(&self) -> Result<(), Box<Diagnostic>> {
        if self.statement.trim().is_empty() {
            return Err(Box::new(Diagnostic::error(
                "XUNLIE-REQ-EMPTY-STATEMENT",
                format!("requirement `{}` has an empty statement", self.id),
            )));
        }
        if self.attributes.keys().any(|key| key.trim().is_empty()) {
            return Err(Box::new(Diagnostic::error(
                "XUNLIE-REQ-EMPTY-ATTRIBUTE",
                format!("requirement `{}` has an empty attribute name", self.id),
            )));
        }
        Ok(())
    }
}

/// Identity and exact-byte digest of an ingested source.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SourceRecord {
    /// Caller-supplied logical source identity.
    pub identity: SourceIdentity,
    /// SHA-256 over exact input bytes, before parsing.
    pub digest: Sha256Digest,
    /// Explicit zero-based source position.
    pub position: usize,
}

impl SourceRecord {
    /// Converts the source record to a location for a specific operation.
    #[must_use]
    pub fn location(&self, operation_position: usize) -> SourceLocation {
        SourceLocation::source(self.identity.clone(), self.digest.clone(), self.position)
            .at_operation(operation_position)
    }
}

/// Producer identity recorded with a contract artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Producer {
    /// Producer implementation name.
    pub name: String,
    /// Producer SemVer or build identity.
    pub version: String,
}

/// Non-semantic metadata of a contract artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContractMetadata {
    /// Implementation that compiled the contract.
    pub producer: Producer,
    /// Optional caller-supplied RFC3339 timestamp. The deterministic core never reads a clock.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl Default for ContractMetadata {
    fn default() -> Self {
        Self {
            producer: Producer {
                name: "xunlie-engine".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            created_at: None,
        }
    }
}

/// Canonical, versioned representation of the effective contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContractIr {
    schema_version: String,
    producer: Producer,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    content_digest: Sha256Digest,
    artifact_digest: Sha256Digest,
    sources: Vec<SourceRecord>,
    resolution_policy: ResolutionPolicy,
    requirements: BTreeMap<RequirementId, Requirement>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticContract<'a> {
    schema_version: &'static str,
    resolution_policy: ResolutionPolicy,
    requirements: &'a BTreeMap<RequirementId, Requirement>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactContract<'a> {
    schema_version: &'a str,
    producer: &'a Producer,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<&'a str>,
    content_digest: &'a Sha256Digest,
    sources: &'a [SourceRecord],
    resolution_policy: ResolutionPolicy,
    requirements: &'a BTreeMap<RequirementId, Requirement>,
}

impl ContractIr {
    /// Creates a validated IR and derives its digest from semantic content.
    pub fn new(
        metadata: ContractMetadata,
        mut sources: Vec<SourceRecord>,
        resolution_policy: ResolutionPolicy,
        requirements: BTreeMap<RequirementId, Requirement>,
    ) -> Result<Self, Vec<Diagnostic>> {
        sources.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then_with(|| left.identity.cmp(&right.identity))
        });
        let content_digest =
            semantic_digest(resolution_policy, &requirements).map_err(|error| {
                vec![Diagnostic::error(
                    "XUNLIE-CANONICAL-SERIALIZE",
                    error.to_string(),
                )]
            })?;
        let artifact_digest = artifact_digest(
            CONTRACT_SCHEMA_VERSION,
            &metadata.producer,
            metadata.created_at.as_deref(),
            &content_digest,
            &sources,
            resolution_policy,
            &requirements,
        )
        .map_err(|error| {
            vec![Diagnostic::error(
                "XUNLIE-CANONICAL-SERIALIZE",
                error.to_string(),
            )]
        })?;
        let contract = Self {
            schema_version: CONTRACT_SCHEMA_VERSION.to_owned(),
            producer: metadata.producer,
            created_at: metadata.created_at,
            content_digest,
            artifact_digest,
            sources,
            resolution_policy,
            requirements,
        };
        contract.validate()?;
        Ok(contract)
    }

    /// Validates schema identity, provenance, requirements, and semantic digest.
    pub fn validate(&self) -> Result<(), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != CONTRACT_SCHEMA_VERSION {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CONTRACT-SCHEMA",
                format!(
                    "unsupported ContractIR schema `{}`; expected `{CONTRACT_SCHEMA_VERSION}`",
                    self.schema_version
                ),
            ));
        }
        if self.producer.name.trim().is_empty() || self.producer.version.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CONTRACT-PRODUCER",
                "producer name and version must not be empty",
            ));
        }
        if self.sources.is_empty() {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CONTRACT-NO-SOURCES",
                "ContractIR must retain at least one source record",
            ));
        }

        let mut identities = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for source in &self.sources {
            if !identities.insert(source.identity.clone()) {
                diagnostics.push(
                    Diagnostic::error(
                        "XUNLIE-SOURCE-DUPLICATE-IDENTITY",
                        format!(
                            "source identity `{}` appears more than once",
                            source.identity
                        ),
                    )
                    .with_primary(source.location(0)),
                );
            }
            if !positions.insert(source.position) {
                diagnostics.push(
                    Diagnostic::error(
                        "XUNLIE-SOURCE-DUPLICATE-POSITION",
                        format!("source position {} appears more than once", source.position),
                    )
                    .with_primary(source.location(0)),
                );
            }
        }
        if self.sources.windows(2).any(|pair| {
            (pair[0].position, &pair[0].identity) > (pair[1].position, &pair[1].identity)
        }) {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-SOURCE-NONCANONICAL-ORDER",
                "sources must be ordered by position and then identity",
            ));
        }

        for (id, requirement) in &self.requirements {
            if id != requirement.id() {
                diagnostics.push(Diagnostic::error(
                    "XUNLIE-CONTRACT-KEY-MISMATCH",
                    format!(
                        "requirement map key `{id}` does not match embedded id `{}`",
                        requirement.id()
                    ),
                ));
            }
            if let Err(diagnostic) = requirement.validate() {
                diagnostics.push(*diagnostic);
            }
        }

        match semantic_digest(self.resolution_policy, &self.requirements) {
            Ok(expected) if expected != self.content_digest => diagnostics.push(Diagnostic::error(
                "XUNLIE-CONTRACT-DIGEST-MISMATCH",
                format!(
                    "content digest is `{}` but canonical content hashes to `{expected}`",
                    self.content_digest
                ),
            )),
            Err(error) => diagnostics.push(Diagnostic::error(
                "XUNLIE-CANONICAL-SERIALIZE",
                error.to_string(),
            )),
            Ok(_) => {}
        }
        match artifact_digest(
            &self.schema_version,
            &self.producer,
            self.created_at.as_deref(),
            &self.content_digest,
            &self.sources,
            self.resolution_policy,
            &self.requirements,
        ) {
            Ok(expected) if expected != self.artifact_digest => {
                diagnostics.push(Diagnostic::error(
                    "XUNLIE-CONTRACT-ARTIFACT-DIGEST-MISMATCH",
                    format!(
                        "artifact digest is `{}` but canonical artifact hashes to `{expected}`",
                        self.artifact_digest
                    ),
                ))
            }
            Err(error) => diagnostics.push(Diagnostic::error(
                "XUNLIE-CANONICAL-SERIALIZE",
                error.to_string(),
            )),
            Ok(_) => {}
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    /// Returns compact canonical JSON for the complete persisted artifact.
    pub fn canonical_json(&self) -> Result<String, CanonicalizationError> {
        to_canonical_json(self)
    }

    /// Returns compact canonical JSON of the semantic payload used for hashing.
    pub fn semantic_canonical_json(&self) -> Result<String, CanonicalizationError> {
        semantic_json(self.resolution_policy, &self.requirements)
    }

    /// Returns the canonical semantic content digest.
    #[must_use]
    pub const fn content_digest(&self) -> &Sha256Digest {
        &self.content_digest
    }

    /// Returns the digest over the full artifact except this digest field itself.
    #[must_use]
    pub const fn artifact_digest(&self) -> &Sha256Digest {
        &self.artifact_digest
    }

    /// Returns the effective requirements in stable identifier order.
    #[must_use]
    pub const fn requirements(&self) -> &BTreeMap<RequirementId, Requirement> {
        &self.requirements
    }

    /// Returns provenance records in explicit source order.
    #[must_use]
    pub fn sources(&self) -> &[SourceRecord] {
        &self.sources
    }

    /// Returns the policy under which history was resolved.
    #[must_use]
    pub const fn resolution_policy(&self) -> ResolutionPolicy {
        self.resolution_policy
    }

    /// Returns the schema identifier.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

fn semantic_json(
    resolution_policy: ResolutionPolicy,
    requirements: &BTreeMap<RequirementId, Requirement>,
) -> Result<String, CanonicalizationError> {
    to_canonical_json(&SemanticContract {
        schema_version: CONTRACT_SCHEMA_VERSION,
        resolution_policy,
        requirements,
    })
}

fn semantic_digest(
    resolution_policy: ResolutionPolicy,
    requirements: &BTreeMap<RequirementId, Requirement>,
) -> Result<Sha256Digest, CanonicalizationError> {
    Ok(Sha256Digest::of_bytes(semantic_json(
        resolution_policy,
        requirements,
    )?))
}

#[allow(clippy::too_many_arguments)]
fn artifact_digest(
    schema_version: &str,
    producer: &Producer,
    created_at: Option<&str>,
    content_digest: &Sha256Digest,
    sources: &[SourceRecord],
    resolution_policy: ResolutionPolicy,
    requirements: &BTreeMap<RequirementId, Requirement>,
) -> Result<Sha256Digest, CanonicalizationError> {
    Ok(Sha256Digest::of_bytes(to_canonical_json(
        &ArtifactContract {
            schema_version,
            producer,
            created_at,
            content_digest,
            sources,
            resolution_policy,
            requirements,
        },
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_requirement_id() {
        assert_eq!(
            RequirementId::new("REQ WITH SPACE"),
            Err(RequirementIdError::InvalidCharacter)
        );
    }

    #[test]
    fn requirement_statement_cannot_be_blank() {
        let error = Requirement::new(
            RequirementId::new("REQ-1").unwrap(),
            RequirementKind::Functional,
            RequirementPriority::Must,
            "  ",
        )
        .unwrap_err();
        assert_eq!(error.code, "XUNLIE-REQ-EMPTY-STATEMENT");
    }

    #[test]
    fn contract_requires_at_least_one_source() {
        let diagnostics = ContractIr::new(
            ContractMetadata::default(),
            Vec::new(),
            ResolutionPolicy::Strict,
            BTreeMap::new(),
        )
        .unwrap_err();

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "XUNLIE-CONTRACT-NO-SOURCES")
        );
    }
}
