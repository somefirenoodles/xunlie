//! Deterministic ingestion and compilation of Xunlie source documents.

#![forbid(unsafe_code)]

use core::fmt;

use serde::Deserialize;
use thiserror::Error;
use xunlie_domain::{
    ContractIr, ContractMetadata, Diagnostic, History, HistoryEvent, Operation, Precedence,
    ResolutionPolicy, Sha256Digest, SourceIdentity, SourceLocation, SourceRecord, resolve_history,
};

/// Schema identifier accepted by the M1 JSON source compiler.
pub const SOURCE_SCHEMA_VERSION: &str = "xunlie.source/v1";

/// Exact source content plus caller-controlled identity and history position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDocument {
    identity: String,
    position: usize,
    source: String,
}

impl SourceDocument {
    /// Creates a source document without performing IO or implicit normalization.
    ///
    /// Validation happens during compilation so all input failures share the
    /// structured `CompileError` surface.
    #[must_use]
    pub fn new(identity: &str, position: usize, source: &str) -> Self {
        Self {
            identity: identity.to_owned(),
            position,
            source: source.to_owned(),
        }
    }

    /// Returns the logical source identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Returns the explicit source position.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the exact input text whose bytes are digested.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Caller-supplied deterministic compilation settings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompileOptions {
    /// Conflict behavior recorded in the resulting IR.
    pub resolution_policy: ResolutionPolicy,
    /// Producer and optional caller-provided timestamp.
    pub metadata: ContractMetadata,
}

/// Compilation failure containing diagnostics and deliberately no partial IR.
#[derive(Clone, Debug, Error)]
#[error("contract compilation failed with {} diagnostic(s)", .diagnostics.len())]
pub struct CompileError {
    diagnostics: Vec<Diagnostic>,
}

impl CompileError {
    /// Returns every structured diagnostic in deterministic source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes the error and returns its diagnostics.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

/// Compiles one in-memory JSON source using a deterministic conventional identity.
pub fn compile(source: &str) -> Result<ContractIr, CompileError> {
    compile_sources(vec![SourceDocument::new("memory://stdin", 0, source)])
}

/// Compiles positioned sources into a canonical `ContractIr` under strict policy.
pub fn compile_sources(sources: Vec<SourceDocument>) -> Result<ContractIr, CompileError> {
    compile_sources_with_options(sources, CompileOptions::default())
}

/// Compiles sources using explicit metadata and resolution policy.
pub fn compile_sources_with_options(
    sources: Vec<SourceDocument>,
    options: CompileOptions,
) -> Result<ContractIr, CompileError> {
    if sources.is_empty() {
        return Err(CompileError {
            diagnostics: vec![Diagnostic::error(
                "XUNLIE-SOURCE-EMPTY-SET",
                "at least one source document is required",
            )],
        });
    }

    let mut ordered = sources;
    ordered.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then_with(|| left.identity.cmp(&right.identity))
    });

    let mut diagnostics = Vec::new();
    let mut records = Vec::with_capacity(ordered.len());
    let mut events = Vec::new();

    for document in &ordered {
        let digest = Sha256Digest::of_bytes(document.source.as_bytes());
        let identity = match SourceIdentity::new(document.identity.clone()) {
            Ok(identity) => identity,
            Err(error) => {
                diagnostics.push(Diagnostic::error(
                    "XUNLIE-SOURCE-INVALID-IDENTITY",
                    format!("source at position {}: {error}", document.position),
                ));
                continue;
            }
        };
        let record = SourceRecord {
            identity,
            digest,
            position: document.position,
        };

        let envelope: SourceEnvelope = match serde_json::from_str(&document.source) {
            Ok(envelope) => envelope,
            Err(error) => {
                diagnostics.push(
                    Diagnostic::error(
                        "XUNLIE-SOURCE-INVALID-JSON",
                        format!("source is not valid {SOURCE_SCHEMA_VERSION} JSON: {error}"),
                    )
                    .with_primary(
                        SourceLocation::source(
                            record.identity.clone(),
                            record.digest.clone(),
                            record.position,
                        )
                        .at_text(error.line(), error.column()),
                    ),
                );
                records.push(record);
                continue;
            }
        };

        if envelope.schema_version != SOURCE_SCHEMA_VERSION {
            diagnostics.push(
                Diagnostic::error(
                    "XUNLIE-SOURCE-UNSUPPORTED-SCHEMA",
                    format!(
                        "source schema `{}` is unsupported; expected `{SOURCE_SCHEMA_VERSION}`",
                        envelope.schema_version
                    ),
                )
                .with_primary(SourceLocation::source(
                    record.identity.clone(),
                    record.digest.clone(),
                    record.position,
                )),
            );
            records.push(record);
            continue;
        }

        for (operation_position, operation) in envelope.operations.into_iter().enumerate() {
            let location = record.location(operation_position);
            events.push(HistoryEvent {
                precedence: Precedence {
                    source_position: record.position,
                    operation_position,
                },
                location,
                operation,
            });
        }
        records.push(record);
    }

    if !diagnostics.is_empty() {
        return Err(CompileError { diagnostics });
    }

    let history = History {
        policy: options.resolution_policy,
        sources: records.clone(),
        events,
    };
    let resolved = resolve_history(&history).map_err(|failure| CompileError {
        diagnostics: failure.diagnostics(),
    })?;

    ContractIr::new(
        options.metadata,
        records,
        options.resolution_policy,
        resolved.into_requirements(),
    )
    .map_err(|diagnostics| CompileError { diagnostics })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SourceEnvelope {
    schema_version: String,
    #[serde(default)]
    operations: Vec<Operation>,
}

impl fmt::Display for SourceDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.identity, self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xunlie_domain::{ContractIr, RequirementId};

    const ADD: &str = r#"{
        "schemaVersion": "xunlie.source/v1",
        "operations": [{
            "op": "add",
            "requirement": {
                "id": "REQ-F-001",
                "kind": "functional",
                "priority": "must",
                "statement": "The compiler preserves provenance."
            }
        }]
    }"#;

    #[test]
    fn compiles_a_valid_source_with_provenance() {
        let contract = compile(ADD).unwrap();
        assert_eq!(contract.schema_version(), "xunlie.contract/v1");
        assert_eq!(contract.sources().len(), 1);
        assert_eq!(contract.sources()[0].identity.as_str(), "memory://stdin");
        assert_eq!(contract.requirements().len(), 1);
        assert_eq!(
            contract.requirements()[&RequirementId::new("REQ-F-001").unwrap()].statement(),
            "The compiler preserves provenance."
        );
        contract.validate().unwrap();
    }

    #[test]
    fn invalid_json_returns_location_and_no_contract() {
        let error = compile("{not json}").unwrap_err();
        assert_eq!(error.diagnostics().len(), 1);
        assert_eq!(error.diagnostics()[0].code, "XUNLIE-SOURCE-INVALID-JSON");
        let location = error.diagnostics()[0].primary.as_ref().unwrap();
        assert_eq!(location.source_position, 0);
        assert!(location.line.is_some());
        assert!(location.column.is_some());
    }

    #[test]
    fn contradictory_adds_fail_with_both_source_locations() {
        let conflicting = ADD.replace(
            "The compiler preserves provenance.",
            "The compiler may discard provenance.",
        );
        let error = compile_sources(vec![
            SourceDocument::new("memory://first", 0, ADD),
            SourceDocument::new("memory://second", 1, &conflicting),
        ])
        .unwrap_err();

        assert_eq!(
            error.diagnostics()[0].code,
            "XUNLIE-RESOLVE-CONTRADICTORY-ADD"
        );
        assert_eq!(error.diagnostics()[0].related.len(), 1);
        assert_eq!(
            error.diagnostics()[0]
                .primary
                .as_ref()
                .unwrap()
                .identity
                .as_str(),
            "memory://second"
        );
    }

    #[test]
    fn semantic_whitespace_does_not_change_contract_digest() {
        let compact = r#"{"schemaVersion":"xunlie.source/v1","operations":[{"op":"add","requirement":{"id":"REQ-F-001","kind":"functional","priority":"must","statement":"The compiler preserves provenance."}}]}"#;
        let pretty = compile(ADD).unwrap();
        let minified = compile(compact).unwrap();

        assert_ne!(pretty.sources()[0].digest, minified.sources()[0].digest);
        assert_eq!(pretty.content_digest(), minified.content_digest());
        assert_ne!(pretty.artifact_digest(), minified.artifact_digest());
        assert_eq!(
            pretty.semantic_canonical_json().unwrap(),
            minified.semantic_canonical_json().unwrap()
        );
    }

    #[test]
    fn canonical_json_survives_typed_round_trip() {
        let original = compile(ADD).unwrap().canonical_json().unwrap();
        let decoded: ContractIr = serde_json::from_str(&original).unwrap();

        decoded.validate().unwrap();
        assert_eq!(decoded.canonical_json().unwrap(), original);
    }

    #[test]
    fn tampered_provenance_invalidates_artifact_digest_only() {
        let original = compile(ADD).unwrap();
        let original_content_digest = original.content_digest().clone();
        let original_artifact_digest = original.artifact_digest().clone();
        let mut value: serde_json::Value =
            serde_json::from_str(&original.canonical_json().unwrap()).unwrap();
        value["sources"][0]["identity"] = serde_json::Value::String("memory://tampered".to_owned());

        let tampered: ContractIr = serde_json::from_value(value).unwrap();
        assert_eq!(tampered.content_digest(), &original_content_digest);
        assert_eq!(tampered.artifact_digest(), &original_artifact_digest);
        let diagnostics = tampered.validate().unwrap_err();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            "XUNLIE-CONTRACT-ARTIFACT-DIGEST-MISMATCH"
        );
    }

    #[test]
    fn tampered_artifact_digest_is_rejected() {
        let original = compile(ADD).unwrap();
        let mut value: serde_json::Value =
            serde_json::from_str(&original.canonical_json().unwrap()).unwrap();
        value["artifactDigest"] = serde_json::Value::String(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
        );

        let tampered: ContractIr = serde_json::from_value(value).unwrap();
        let diagnostics = tampered.validate().unwrap_err();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            "XUNLIE-CONTRACT-ARTIFACT-DIGEST-MISMATCH"
        );
    }
}
