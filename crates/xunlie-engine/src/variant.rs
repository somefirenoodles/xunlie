use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use xunlie_domain::{
    CertifiedHistory, Diagnostic, EquivalenceCertificate, PreconditionEvaluation,
    PreconditionStatus, Producer, Sha256Digest, VariantOperatorIdentity,
};

use crate::{SourceDocument, compile_sources};

/// Schema identifier for the persisted source history and certificate bundle.
pub const CERTIFIED_VARIANT_SCHEMA_VERSION: &str = "xunlie.certified-variant/v1";
const HISTORY_FINGERPRINT_SCHEMA_VERSION: &str = "xunlie.source-history/v1";
/// Stable identity of [`JsonNormalizationOperator`].
pub const JSON_NORMALIZATION_OPERATOR_ID: &str = "json.presentation.normalize";
/// Stable identity of [`ReverseIndependentAddsOperator`].
pub const REVERSE_INDEPENDENT_ADDS_OPERATOR_ID: &str = "history.independent-adds.reverse";
/// Version shared by the initial built-in operator implementations.
pub const BUILTIN_OPERATOR_VERSION: &str = "1.0.0";

/// Stable identifiers accepted by [`generate_builtin_variant`].
pub const BUILTIN_OPERATOR_IDS: [&str; 2] = [
    JSON_NORMALIZATION_OPERATOR_ID,
    REVERSE_INDEPENDENT_ADDS_OPERATOR_ID,
];

/// A pure history transformation with explicit, executable preconditions.
///
/// Implementations may propose a transformation, but cannot certify themselves:
/// [`generate_certified_variant`] recompiles both sides and fails closed when
/// their semantic contract digests differ.
pub trait VariantOperator: fmt::Debug {
    /// Returns the stable operator id and implementation version.
    fn identity(&self) -> VariantOperatorIdentity;

    /// Evaluates the operator's complete safe-domain preconditions.
    fn evaluate(&self, sources: &[SourceDocument]) -> Vec<PreconditionEvaluation>;

    /// Applies the transformation after every precondition passed.
    fn transform(&self, sources: &[SourceDocument]) -> Result<Vec<SourceDocument>, String>;
}

/// Canonicalizes the JSON presentation of every source without changing its data model.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonNormalizationOperator;

impl VariantOperator for JsonNormalizationOperator {
    fn identity(&self) -> VariantOperatorIdentity {
        VariantOperatorIdentity::new(JSON_NORMALIZATION_OPERATOR_ID, BUILTIN_OPERATOR_VERSION)
    }

    fn evaluate(&self, sources: &[SourceDocument]) -> Vec<PreconditionEvaluation> {
        let all_valid = sources
            .iter()
            .all(|source| serde_json::from_str::<serde_json::Value>(source.source()).is_ok());
        vec![if all_valid {
            PreconditionEvaluation::passed(
                "json.normalize.all-sources-valid-json",
                format!("all {} source document(s) parsed as JSON", sources.len()),
            )
        } else {
            PreconditionEvaluation::failed(
                "json.normalize.all-sources-valid-json",
                "at least one source document is not valid JSON",
            )
        }]
    }

    fn transform(&self, sources: &[SourceDocument]) -> Result<Vec<SourceDocument>, String> {
        sources
            .iter()
            .map(|source| {
                let value: serde_json::Value = serde_json::from_str(source.source())
                    .map_err(|error| format!("could not parse {}: {error}", source.identity()))?;
                let normalized = serde_json::to_string(&value).map_err(|error| {
                    format!("could not normalize {}: {error}", source.identity())
                })?;
                Ok(SourceDocument::new(
                    source.identity(),
                    source.position(),
                    &normalized,
                ))
            })
            .collect()
    }
}

/// Reverses a history made exclusively of independent additions.
///
/// Distinct requirement ids make the operations commutative under strict
/// resolution. Replacements and revocations are deliberately excluded.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReverseIndependentAddsOperator;

impl VariantOperator for ReverseIndependentAddsOperator {
    fn identity(&self) -> VariantOperatorIdentity {
        VariantOperatorIdentity::new(
            REVERSE_INDEPENDENT_ADDS_OPERATOR_ID,
            BUILTIN_OPERATOR_VERSION,
        )
    }

    fn evaluate(&self, sources: &[SourceDocument]) -> Vec<PreconditionEvaluation> {
        let exactly_one = sources.len() == 1;
        let parsed = sources.first().and_then(|source| {
            serde_json::from_str::<OperatorSourceEnvelope>(source.source()).ok()
        });
        let at_least_two = parsed
            .as_ref()
            .is_some_and(|source| source.operations.len() >= 2);
        let all_adds = parsed.as_ref().is_some_and(|source| {
            source
                .operations
                .iter()
                .all(|operation| matches!(operation, xunlie_domain::Operation::Add { .. }))
        });
        let distinct_ids = parsed.as_ref().is_some_and(|source| {
            let ids: BTreeSet<_> = source
                .operations
                .iter()
                .filter_map(|operation| match operation {
                    xunlie_domain::Operation::Add { requirement } => Some(requirement.id()),
                    xunlie_domain::Operation::Replace { .. }
                    | xunlie_domain::Operation::Revoke { .. } => None,
                })
                .collect();
            ids.len() == source.operations.len()
        });

        vec![
            evaluation(
                "independent-adds.single-source",
                exactly_one,
                "history contains exactly one source document",
                format!(
                    "operator requires exactly one source document; found {}",
                    sources.len()
                ),
            ),
            evaluation(
                "independent-adds.supported-envelope",
                parsed.is_some(),
                "source uses the supported xunlie.source/v1 envelope",
                "source is not a supported xunlie.source/v1 envelope",
            ),
            evaluation(
                "independent-adds.minimum-cardinality",
                at_least_two,
                "history contains at least two operations",
                "at least two operations are required to reverse a history",
            ),
            evaluation(
                "independent-adds.add-only",
                all_adds,
                "every operation is an addition",
                "replacement and revocation operations are order-dependent and excluded",
            ),
            evaluation(
                "independent-adds.distinct-targets",
                distinct_ids,
                "every addition targets a distinct requirement id",
                "duplicate requirement ids are not independent and are excluded",
            ),
        ]
    }

    fn transform(&self, sources: &[SourceDocument]) -> Result<Vec<SourceDocument>, String> {
        let [source] = sources else {
            return Err("reverse-independent-adds requires exactly one source".to_owned());
        };
        let mut envelope: OperatorSourceEnvelope = serde_json::from_str(source.source())
            .map_err(|error| format!("could not parse {}: {error}", source.identity()))?;
        envelope.operations.reverse();
        let transformed = serde_json::to_string(&envelope)
            .map_err(|error| format!("could not serialize transformed history: {error}"))?;
        Ok(vec![SourceDocument::new(
            source.identity(),
            source.position(),
            &transformed,
        )])
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct OperatorSourceEnvelope {
    schema_version: String,
    #[serde(default)]
    operations: Vec<xunlie_domain::Operation>,
}

/// A successfully transformed history and its independently checkable certificate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CertifiedVariant {
    schema_version: String,
    producer: Producer,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    content_digest: Sha256Digest,
    sources: Vec<SourceDocument>,
    certificate: EquivalenceCertificate,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CertifiedVariantPayload<'a> {
    schema_version: &'a str,
    producer: &'a Producer,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<&'a str>,
    sources: &'a [SourceDocument],
    certificate: &'a EquivalenceCertificate,
}

impl CertifiedVariant {
    fn new(
        sources: Vec<SourceDocument>,
        certificate: EquivalenceCertificate,
    ) -> Result<Self, VariantError> {
        let producer = Producer {
            name: "xunlie-engine".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        };
        let content_digest = certified_variant_digest(
            CERTIFIED_VARIANT_SCHEMA_VERSION,
            &producer,
            None,
            &sources,
            &certificate,
        )?;
        let artifact = Self {
            schema_version: CERTIFIED_VARIANT_SCHEMA_VERSION.to_owned(),
            producer,
            created_at: None,
            content_digest,
            sources,
            certificate,
        };
        artifact.validate()?;
        Ok(artifact)
    }

    /// Validates container integrity and recompiles its transformed history.
    pub fn validate(&self) -> Result<(), VariantError> {
        if self.schema_version != CERTIFIED_VARIANT_SCHEMA_VERSION {
            return Err(VariantError::new(
                "XUNLIE-VARIANT-SCHEMA",
                format!(
                    "unsupported variant schema `{}`; expected `{CERTIFIED_VARIANT_SCHEMA_VERSION}`",
                    self.schema_version
                ),
            ));
        }
        if self.producer.name.trim().is_empty() || self.producer.version.trim().is_empty() {
            return Err(VariantError::new(
                "XUNLIE-VARIANT-PRODUCER",
                "variant producer name and version must not be empty",
            ));
        }
        self.certificate.validate().map_err(|diagnostics| {
            VariantError::from_diagnostics(
                "XUNLIE-VARIANT-CERTIFICATE-INVALID",
                "equivalence certificate failed validation",
                diagnostics,
            )
        })?;
        let expected_digest = certified_variant_digest(
            &self.schema_version,
            &self.producer,
            self.created_at.as_deref(),
            &self.sources,
            &self.certificate,
        )?;
        if self.content_digest != expected_digest {
            return Err(VariantError::new(
                "XUNLIE-VARIANT-DIGEST-MISMATCH",
                format!(
                    "variant content digest is `{}` but canonical content hashes to `{expected_digest}`",
                    self.content_digest
                ),
            ));
        }
        let observed_history = history_digest(&self.sources)?;
        if observed_history != self.certificate.after().history_digest {
            return Err(VariantError::new(
                "XUNLIE-VARIANT-HISTORY-DIGEST-MISMATCH",
                format!(
                    "variant history hashes to `{observed_history}` but certificate records `{}`",
                    self.certificate.after().history_digest
                ),
            ));
        }
        let contract = compile_sources(self.sources.clone()).map_err(|error| {
            VariantError::from_diagnostics(
                "XUNLIE-VARIANT-INVALID-OUTPUT",
                "persisted variant history did not compile",
                error.into_diagnostics(),
            )
        })?;
        if contract.content_digest() != &self.certificate.after().content_digest
            || contract.artifact_digest() != &self.certificate.after().artifact_digest
        {
            return Err(VariantError::new(
                "XUNLIE-VARIANT-CONTRACT-DIGEST-MISMATCH",
                "compiled variant contract does not match the certificate after-state",
            ));
        }
        Ok(())
    }

    /// Returns compact deterministic JSON for persistence.
    pub fn canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Returns the transformed source history.
    #[must_use]
    pub fn sources(&self) -> &[SourceDocument] {
        &self.sources
    }

    /// Returns the equivalence evidence bound to this history.
    #[must_use]
    pub const fn certificate(&self) -> &EquivalenceCertificate {
        &self.certificate
    }

    /// Returns the digest over the complete bundle except this field itself.
    #[must_use]
    pub const fn content_digest(&self) -> &Sha256Digest {
        &self.content_digest
    }

    /// Returns the persisted artifact schema.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

/// Safe-domain exclusion with every evaluated reason retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExcludedVariant {
    operator: VariantOperatorIdentity,
    preconditions: Vec<PreconditionEvaluation>,
}

impl ExcludedVariant {
    /// Returns the operator that could not safely transform the input.
    #[must_use]
    pub const fn operator(&self) -> &VariantOperatorIdentity {
        &self.operator
    }

    /// Returns passed and failed checks; failed checks explain the exclusion.
    #[must_use]
    pub fn preconditions(&self) -> &[PreconditionEvaluation] {
        &self.preconditions
    }
}

/// Result of requesting a certified variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariantGeneration {
    /// The transformed history passed independent equivalence verification.
    Certified(Box<CertifiedVariant>),
    /// One or more executable preconditions excluded the candidate.
    Excluded(ExcludedVariant),
}

/// Fail-closed variant generation or verification error.
#[derive(Clone, Debug, Error)]
#[error("{message}")]
pub struct VariantError {
    code: &'static str,
    message: String,
    diagnostics: Vec<Diagnostic>,
}

impl VariantError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            code,
            diagnostics: vec![Diagnostic::error(code, message.clone())],
            message,
        }
    }

    fn from_diagnostics(
        code: &'static str,
        message: impl Into<String>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            diagnostics,
        }
    }

    /// Returns the stable failure classification.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Returns structured diagnostics without any partial certified artifact.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

/// Generates a variant with one registered built-in operator.
pub fn generate_builtin_variant(
    sources: Vec<SourceDocument>,
    operator_id: &str,
) -> Result<VariantGeneration, VariantError> {
    let operator = builtin_operator(operator_id).ok_or_else(|| {
        VariantError::new(
            "XUNLIE-VARIANT-UNKNOWN-OPERATOR",
            format!(
                "unknown variant operator `{operator_id}`; expected one of {}",
                BUILTIN_OPERATOR_IDS.join(", ")
            ),
        )
    })?;
    generate_certified_variant(sources, operator.as_ref())
}

/// Applies an operator, recompiles both histories, and certifies only equality.
pub fn generate_certified_variant(
    mut sources: Vec<SourceDocument>,
    operator: &dyn VariantOperator,
) -> Result<VariantGeneration, VariantError> {
    sort_sources(&mut sources);
    let identity = operator.identity();
    let baseline = compile_sources(sources.clone()).map_err(|error| {
        VariantError::from_diagnostics(
            "XUNLIE-VARIANT-INVALID-BASELINE",
            "baseline history did not compile",
            error.into_diagnostics(),
        )
    })?;

    let mut preconditions = vec![PreconditionEvaluation::passed(
        "xunlie.variant.baseline-compiles",
        format!(
            "baseline compiled to content digest {}",
            baseline.content_digest()
        ),
    )];
    preconditions.extend(operator.evaluate(&sources));
    if has_failed_precondition(&preconditions) {
        return Ok(VariantGeneration::Excluded(ExcludedVariant {
            operator: identity,
            preconditions,
        }));
    }

    let mut transformed = operator.transform(&sources).map_err(|message| {
        VariantError::new(
            "XUNLIE-VARIANT-OPERATOR-FAILED",
            format!(
                "operator `{}` failed after its preconditions passed: {message}",
                identity.id
            ),
        )
    })?;
    sort_sources(&mut transformed);
    if sources == transformed {
        preconditions.push(PreconditionEvaluation::failed(
            "xunlie.variant.output-differs",
            "operator output is byte-for-byte identical to the baseline history",
        ));
        return Ok(VariantGeneration::Excluded(ExcludedVariant {
            operator: identity,
            preconditions,
        }));
    }
    preconditions.push(PreconditionEvaluation::passed(
        "xunlie.variant.output-differs",
        "operator changed the exact source history representation",
    ));

    let candidate = compile_sources(transformed.clone()).map_err(|error| {
        VariantError::from_diagnostics(
            "XUNLIE-VARIANT-INVALID-OUTPUT",
            format!(
                "operator `{}` produced a history that did not compile",
                identity.id
            ),
            error.into_diagnostics(),
        )
    })?;
    preconditions.push(PreconditionEvaluation::passed(
        "xunlie.variant.output-compiles",
        format!(
            "transformed history compiled to content digest {}",
            candidate.content_digest()
        ),
    ));

    if baseline.content_digest() != candidate.content_digest() {
        return Err(VariantError::new(
            "XUNLIE-VARIANT-NOT-EQUIVALENT",
            format!(
                "operator `{}` changed semantic content from {} to {}",
                identity.id,
                baseline.content_digest(),
                candidate.content_digest()
            ),
        ));
    }
    preconditions.push(PreconditionEvaluation::passed(
        "xunlie.variant.content-digest-equal",
        format!(
            "both histories compile to content digest {}",
            baseline.content_digest()
        ),
    ));

    let before = certified_history(&sources, &baseline)?;
    let after = certified_history(&transformed, &candidate)?;
    let certificate = EquivalenceCertificate::new(
        Producer {
            name: "xunlie-engine".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        identity,
        preconditions,
        before,
        after,
    )
    .map_err(|diagnostics| {
        VariantError::from_diagnostics(
            "XUNLIE-VARIANT-CERTIFICATE-INVALID",
            "generated equivalence evidence did not satisfy certificate invariants",
            diagnostics,
        )
    })?;

    Ok(VariantGeneration::Certified(Box::new(
        CertifiedVariant::new(transformed, certificate)?,
    )))
}

/// Verifies a persisted artifact using the built-in operator named by its certificate.
pub fn verify_certified_variant(
    baseline_sources: Vec<SourceDocument>,
    artifact: &CertifiedVariant,
) -> Result<(), VariantError> {
    let operator = builtin_operator(&artifact.certificate.operator().id).ok_or_else(|| {
        VariantError::new(
            "XUNLIE-VARIANT-UNKNOWN-OPERATOR",
            format!(
                "certificate names unsupported operator `{}`",
                artifact.certificate.operator().id
            ),
        )
    })?;
    verify_certified_variant_with_operator(baseline_sources, artifact, operator.as_ref())
}

/// Replays an operator and compares all persisted source and certificate evidence.
pub fn verify_certified_variant_with_operator(
    baseline_sources: Vec<SourceDocument>,
    artifact: &CertifiedVariant,
    operator: &dyn VariantOperator,
) -> Result<(), VariantError> {
    artifact.validate()?;
    if artifact.certificate.operator() != &operator.identity() {
        return Err(VariantError::new(
            "XUNLIE-VARIANT-OPERATOR-MISMATCH",
            format!(
                "certificate operator {}@{} does not match verifier {}@{}",
                artifact.certificate.operator().id,
                artifact.certificate.operator().version,
                operator.identity().id,
                operator.identity().version
            ),
        ));
    }

    let regenerated = generate_certified_variant(baseline_sources, operator)?;
    let VariantGeneration::Certified(expected) = regenerated else {
        return Err(VariantError::new(
            "XUNLIE-VARIANT-REPLAY-EXCLUDED",
            "operator no longer admits the baseline under its recorded preconditions",
        ));
    };
    if expected.as_ref() != artifact {
        return Err(VariantError::new(
            "XUNLIE-VARIANT-REPLAY-MISMATCH",
            "persisted variant or certificate differs from deterministic operator replay",
        ));
    }
    Ok(())
}

fn certified_history(
    sources: &[SourceDocument],
    contract: &xunlie_domain::ContractIr,
) -> Result<CertifiedHistory, VariantError> {
    Ok(CertifiedHistory {
        history_digest: history_digest(sources)?,
        content_digest: contract.content_digest().clone(),
        artifact_digest: contract.artifact_digest().clone(),
    })
}

fn certified_variant_digest(
    schema_version: &str,
    producer: &Producer,
    created_at: Option<&str>,
    sources: &[SourceDocument],
    certificate: &EquivalenceCertificate,
) -> Result<Sha256Digest, VariantError> {
    let encoded = serde_json::to_vec(&CertifiedVariantPayload {
        schema_version,
        producer,
        created_at,
        sources,
        certificate,
    })
    .map_err(|error| {
        VariantError::new(
            "XUNLIE-VARIANT-CANONICAL-SERIALIZE",
            format!("could not serialize certified variant: {error}"),
        )
    })?;
    Ok(Sha256Digest::of_bytes(encoded))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryFingerprint<'a> {
    schema_version: &'static str,
    sources: Vec<&'a SourceDocument>,
}

fn history_digest(sources: &[SourceDocument]) -> Result<Sha256Digest, VariantError> {
    let mut ordered: Vec<_> = sources.iter().collect();
    ordered.sort_by(|left, right| {
        left.position()
            .cmp(&right.position())
            .then_with(|| left.identity().cmp(right.identity()))
    });
    let encoded = serde_json::to_vec(&HistoryFingerprint {
        schema_version: HISTORY_FINGERPRINT_SCHEMA_VERSION,
        sources: ordered,
    })
    .map_err(|error| {
        VariantError::new(
            "XUNLIE-VARIANT-FINGERPRINT",
            format!("could not fingerprint source history: {error}"),
        )
    })?;
    Ok(Sha256Digest::of_bytes(encoded))
}

fn sort_sources(sources: &mut [SourceDocument]) {
    sources.sort_by(|left, right| {
        left.position()
            .cmp(&right.position())
            .then_with(|| left.identity().cmp(right.identity()))
    });
}

fn builtin_operator(id: &str) -> Option<Box<dyn VariantOperator>> {
    match id {
        JSON_NORMALIZATION_OPERATOR_ID => Some(Box::new(JsonNormalizationOperator)),
        REVERSE_INDEPENDENT_ADDS_OPERATOR_ID => Some(Box::new(ReverseIndependentAddsOperator)),
        _ => None,
    }
}

fn has_failed_precondition(preconditions: &[PreconditionEvaluation]) -> bool {
    preconditions
        .iter()
        .any(|item| item.status == PreconditionStatus::Failed)
}

fn evaluation(
    id: &'static str,
    passed: bool,
    success: impl Into<String>,
    failure: impl Into<String>,
) -> PreconditionEvaluation {
    if passed {
        PreconditionEvaluation::passed(id, success)
    } else {
        PreconditionEvaluation::failed(id, failure)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const PRETTY_SOURCE: &str = r#"{
        "schemaVersion": "xunlie.source/v1",
        "operations": [{
            "op": "add",
            "requirement": {
                "id": "REQ-1",
                "kind": "functional",
                "priority": "must",
                "statement": "Preserve provenance."
            }
        }]
    }"#;

    fn source(text: &str) -> Vec<SourceDocument> {
        vec![SourceDocument::new("memory://source", 0, text)]
    }

    fn two_adds(first: &str, second: &str) -> String {
        format!(
            r#"{{"schemaVersion":"xunlie.source/v1","operations":[{{"op":"add","requirement":{{"id":"REQ-1","kind":"functional","priority":"must","statement":{first:?}}}}},{{"op":"add","requirement":{{"id":"REQ-2","kind":"quality","priority":"should","statement":{second:?}}}}}]}}"#
        )
    }

    fn certified(result: VariantGeneration) -> CertifiedVariant {
        match result {
            VariantGeneration::Certified(variant) => *variant,
            VariantGeneration::Excluded(exclusion) => {
                panic!("unexpected exclusion: {:?}", exclusion.preconditions())
            }
        }
    }

    #[test]
    fn normalization_is_certified_and_replayable() {
        let baseline = source(PRETTY_SOURCE);
        let variant = certified(
            generate_certified_variant(baseline.clone(), &JsonNormalizationOperator).unwrap(),
        );

        assert_eq!(variant.schema_version(), CERTIFIED_VARIANT_SCHEMA_VERSION);
        assert_eq!(variant.content_digest().as_str().len(), 71);
        variant.validate().unwrap();
        assert_ne!(variant.sources()[0].source(), PRETTY_SOURCE);
        assert_eq!(
            variant.certificate().before().content_digest,
            variant.certificate().after().content_digest
        );
        assert_ne!(
            variant.certificate().before().history_digest,
            variant.certificate().after().history_digest
        );
        assert!(variant.certificate().preconditions().iter().any(|item| {
            item.id == "json.normalize.all-sources-valid-json"
                && item.status == PreconditionStatus::Passed
        }));
        verify_certified_variant(baseline, &variant).unwrap();
    }

    #[test]
    fn repeated_generation_is_byte_deterministic() {
        let first = certified(
            generate_certified_variant(source(PRETTY_SOURCE), &JsonNormalizationOperator).unwrap(),
        );
        let second = certified(
            generate_certified_variant(source(PRETTY_SOURCE), &JsonNormalizationOperator).unwrap(),
        );

        assert_eq!(
            first.canonical_json().unwrap(),
            second.canonical_json().unwrap()
        );
    }

    #[test]
    fn source_storage_order_does_not_change_certified_artifact() {
        let first = SourceDocument::new("memory://first", 0, PRETTY_SOURCE);
        let second_source = PRETTY_SOURCE
            .replace("REQ-1", "REQ-2")
            .replace("Preserve provenance.", "Remain deterministic.");
        let second = SourceDocument::new("memory://second", 1, &second_source);
        let forward = certified(
            generate_certified_variant(
                vec![first.clone(), second.clone()],
                &JsonNormalizationOperator,
            )
            .unwrap(),
        );
        let reverse = certified(
            generate_certified_variant(vec![second, first], &JsonNormalizationOperator).unwrap(),
        );

        assert_eq!(
            forward.canonical_json().unwrap(),
            reverse.canonical_json().unwrap()
        );
    }

    #[test]
    fn already_normalized_json_is_excluded_with_reason() {
        let normalized = serde_json::to_string(
            &serde_json::from_str::<serde_json::Value>(PRETTY_SOURCE).unwrap(),
        )
        .unwrap();
        let result =
            generate_certified_variant(source(&normalized), &JsonNormalizationOperator).unwrap();
        let VariantGeneration::Excluded(exclusion) = result else {
            panic!("canonical input should be excluded")
        };

        assert!(exclusion.preconditions().iter().any(|item| {
            item.id == "xunlie.variant.output-differs" && item.status == PreconditionStatus::Failed
        }));
    }

    #[test]
    fn dependent_history_is_excluded_from_reversal() {
        let dependent = r#"{"schemaVersion":"xunlie.source/v1","operations":[{"op":"add","requirement":{"id":"REQ-1","kind":"functional","priority":"must","statement":"first"}},{"op":"replace","target":"REQ-1","requirement":{"id":"REQ-1","kind":"functional","priority":"must","statement":"second"}}]}"#;
        let result =
            generate_certified_variant(source(dependent), &ReverseIndependentAddsOperator).unwrap();
        let VariantGeneration::Excluded(exclusion) = result else {
            panic!("dependent history should be excluded")
        };

        assert!(exclusion.preconditions().iter().any(|item| {
            item.id == "independent-adds.add-only" && item.status == PreconditionStatus::Failed
        }));
    }

    #[test]
    fn operators_report_invalid_inputs_and_transform_fails_closed() {
        let invalid_json = source("{not-json}");
        let evaluations = JsonNormalizationOperator.evaluate(&invalid_json);
        assert_eq!(evaluations.len(), 1);
        assert_eq!(evaluations[0].status, PreconditionStatus::Failed);
        assert!(JsonNormalizationOperator.transform(&invalid_json).is_err());

        let multiple_sources = vec![
            SourceDocument::new("memory://first", 0, PRETTY_SOURCE),
            SourceDocument::new("memory://second", 1, PRETTY_SOURCE),
        ];
        let evaluations = ReverseIndependentAddsOperator.evaluate(&multiple_sources);
        assert!(evaluations.iter().any(|item| {
            item.id == "independent-adds.single-source" && item.status == PreconditionStatus::Failed
        }));
        assert!(
            ReverseIndependentAddsOperator
                .transform(&multiple_sources)
                .is_err()
        );
        assert!(
            ReverseIndependentAddsOperator
                .transform(&invalid_json)
                .is_err()
        );
    }

    #[test]
    fn unknown_builtin_and_invalid_baseline_are_rejected() {
        let unknown = generate_builtin_variant(source(PRETTY_SOURCE), "test.unknown").unwrap_err();
        assert_eq!(unknown.code(), "XUNLIE-VARIANT-UNKNOWN-OPERATOR");
        assert!(unknown.to_string().contains("test.unknown"));

        let invalid =
            generate_certified_variant(Vec::new(), &JsonNormalizationOperator).unwrap_err();
        assert_eq!(invalid.code(), "XUNLIE-VARIANT-INVALID-BASELINE");
        assert_eq!(invalid.diagnostics()[0].code, "XUNLIE-SOURCE-EMPTY-SET");
    }

    #[derive(Debug)]
    struct TransformFailureOperator;

    impl VariantOperator for TransformFailureOperator {
        fn identity(&self) -> VariantOperatorIdentity {
            VariantOperatorIdentity::new("test.transform-failure", "1.0.0")
        }

        fn evaluate(&self, _sources: &[SourceDocument]) -> Vec<PreconditionEvaluation> {
            vec![PreconditionEvaluation::passed(
                "test.transform-ready",
                "fixture transform may run",
            )]
        }

        fn transform(&self, _sources: &[SourceDocument]) -> Result<Vec<SourceDocument>, String> {
            Err("fixture transform failed".to_owned())
        }
    }

    #[derive(Debug)]
    struct InvalidOutputOperator;

    impl VariantOperator for InvalidOutputOperator {
        fn identity(&self) -> VariantOperatorIdentity {
            VariantOperatorIdentity::new("test.invalid-output", "1.0.0")
        }

        fn evaluate(&self, _sources: &[SourceDocument]) -> Vec<PreconditionEvaluation> {
            vec![PreconditionEvaluation::passed(
                "test.invalid-output-ready",
                "fixture transform may run",
            )]
        }

        fn transform(&self, _sources: &[SourceDocument]) -> Result<Vec<SourceDocument>, String> {
            Ok(source("{not-json}"))
        }
    }

    #[test]
    fn operator_failure_and_invalid_output_have_distinct_errors() {
        let failed = generate_certified_variant(source(PRETTY_SOURCE), &TransformFailureOperator)
            .unwrap_err();
        assert_eq!(failed.code(), "XUNLIE-VARIANT-OPERATOR-FAILED");
        assert!(failed.to_string().contains("fixture transform failed"));

        let invalid =
            generate_certified_variant(source(PRETTY_SOURCE), &InvalidOutputOperator).unwrap_err();
        assert_eq!(invalid.code(), "XUNLIE-VARIANT-INVALID-OUTPUT");
        assert_eq!(invalid.diagnostics()[0].code, "XUNLIE-SOURCE-INVALID-JSON");
    }

    #[test]
    fn invalid_nested_certificate_is_reported_by_the_container() {
        let original = certified(
            generate_certified_variant(source(PRETTY_SOURCE), &JsonNormalizationOperator).unwrap(),
        );
        let mut value: serde_json::Value =
            serde_json::from_str(&original.canonical_json().unwrap()).unwrap();
        value["certificate"]["schemaVersion"] =
            serde_json::Value::String("xunlie.equivalence-certificate/v999".to_owned());
        let invalid: CertifiedVariant = serde_json::from_value(value).unwrap();

        let error = invalid.validate().unwrap_err();
        assert_eq!(error.code(), "XUNLIE-VARIANT-CERTIFICATE-INVALID");
        assert!(
            error
                .diagnostics()
                .iter()
                .any(|item| item.code == "XUNLIE-CERTIFICATE-SCHEMA")
        );
    }

    #[test]
    fn verifier_rejects_unknown_mismatched_excluded_and_divergent_replay() {
        let baseline = source(PRETTY_SOURCE);
        let variant = certified(
            generate_certified_variant(baseline.clone(), &JsonNormalizationOperator).unwrap(),
        );

        let mismatch = verify_certified_variant_with_operator(
            baseline.clone(),
            &variant,
            &ReverseIndependentAddsOperator,
        )
        .unwrap_err();
        assert_eq!(mismatch.code(), "XUNLIE-VARIANT-OPERATOR-MISMATCH");

        let normalized = variant.sources().to_vec();
        let excluded = verify_certified_variant_with_operator(
            normalized,
            &variant,
            &JsonNormalizationOperator,
        )
        .unwrap_err();
        assert_eq!(excluded.code(), "XUNLIE-VARIANT-REPLAY-EXCLUDED");

        let alternate = source(&format!("\n{PRETTY_SOURCE}"));
        let divergent =
            verify_certified_variant_with_operator(alternate, &variant, &JsonNormalizationOperator)
                .unwrap_err();
        assert_eq!(divergent.code(), "XUNLIE-VARIANT-REPLAY-MISMATCH");

        let mut value: serde_json::Value =
            serde_json::from_str(&variant.canonical_json().unwrap()).unwrap();
        value["certificate"]["operator"]["id"] =
            serde_json::Value::String("test.unknown".to_owned());
        let unsupported: CertifiedVariant = serde_json::from_value(value).unwrap();
        let unknown = verify_certified_variant(baseline, &unsupported).unwrap_err();
        assert_eq!(unknown.code(), "XUNLIE-VARIANT-UNKNOWN-OPERATOR");
    }

    #[test]
    fn tampered_variant_fails_deterministic_replay() {
        let baseline = source(PRETTY_SOURCE);
        let original = certified(
            generate_certified_variant(baseline.clone(), &JsonNormalizationOperator).unwrap(),
        );
        let mut value: serde_json::Value =
            serde_json::from_str(&original.canonical_json().unwrap()).unwrap();
        value["sources"][0]["source"] = serde_json::Value::String(PRETTY_SOURCE.to_owned());
        let tampered: CertifiedVariant = serde_json::from_value(value).unwrap();

        let error = verify_certified_variant(baseline, &tampered).unwrap_err();
        assert_eq!(error.code(), "XUNLIE-VARIANT-DIGEST-MISMATCH");
    }

    #[test]
    fn variant_schema_and_producer_fields_are_independently_validated() {
        let original = certified(
            generate_certified_variant(source(PRETTY_SOURCE), &JsonNormalizationOperator).unwrap(),
        );

        let mut wrong_schema = original.clone();
        wrong_schema.schema_version = "xunlie.certified-variant/v999".to_owned();
        assert_eq!(
            wrong_schema.validate().unwrap_err().code(),
            "XUNLIE-VARIANT-SCHEMA"
        );

        let mut blank_name = original.clone();
        blank_name.producer.name.clear();
        let mut blank_version = original;
        blank_version.producer.version.clear();
        for invalid in [blank_name, blank_version] {
            assert_eq!(
                invalid.validate().unwrap_err().code(),
                "XUNLIE-VARIANT-PRODUCER"
            );
        }
    }

    #[test]
    fn source_tampering_is_detected_even_with_rehashed_container() {
        let mut variant = certified(
            generate_certified_variant(source(PRETTY_SOURCE), &JsonNormalizationOperator).unwrap(),
        );
        variant.sources[0] = SourceDocument::new("memory://source", 0, PRETTY_SOURCE);
        variant.content_digest = certified_variant_digest(
            &variant.schema_version,
            &variant.producer,
            variant.created_at.as_deref(),
            &variant.sources,
            &variant.certificate,
        )
        .unwrap();

        assert_eq!(
            variant.validate().unwrap_err().code(),
            "XUNLIE-VARIANT-HISTORY-DIGEST-MISMATCH"
        );
    }

    #[test]
    fn content_and_artifact_mismatches_are_independently_rejected() {
        let original = certified(
            generate_certified_variant(source(PRETTY_SOURCE), &JsonNormalizationOperator).unwrap(),
        );
        let producer = Producer {
            name: "xunlie-engine".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        };

        let mut wrong_artifact_after = original.certificate.after().clone();
        wrong_artifact_after.artifact_digest = Sha256Digest::of_bytes("wrong-after-artifact");
        let wrong_artifact_certificate = EquivalenceCertificate::new(
            producer.clone(),
            original.certificate.operator().clone(),
            original.certificate.preconditions().to_vec(),
            original.certificate.before().clone(),
            wrong_artifact_after,
        )
        .unwrap();

        let wrong_content = Sha256Digest::of_bytes("wrong-contract-content");
        let mut wrong_content_before = original.certificate.before().clone();
        wrong_content_before.content_digest = wrong_content.clone();
        let mut wrong_content_after = original.certificate.after().clone();
        wrong_content_after.content_digest = wrong_content;
        let wrong_content_certificate = EquivalenceCertificate::new(
            producer,
            original.certificate.operator().clone(),
            original.certificate.preconditions().to_vec(),
            wrong_content_before,
            wrong_content_after,
        )
        .unwrap();

        for certificate in [wrong_artifact_certificate, wrong_content_certificate] {
            let mut invalid = original.clone();
            invalid.certificate = certificate;
            invalid.content_digest = certified_variant_digest(
                &invalid.schema_version,
                &invalid.producer,
                invalid.created_at.as_deref(),
                &invalid.sources,
                &invalid.certificate,
            )
            .unwrap();

            assert_eq!(
                invalid.validate().unwrap_err().code(),
                "XUNLIE-VARIANT-CONTRACT-DIGEST-MISMATCH"
            );
        }
    }

    #[derive(Debug)]
    struct SemanticMutationOperator;

    impl VariantOperator for SemanticMutationOperator {
        fn identity(&self) -> VariantOperatorIdentity {
            VariantOperatorIdentity::new("test.semantic-mutation", "1.0.0")
        }

        fn evaluate(&self, _sources: &[SourceDocument]) -> Vec<PreconditionEvaluation> {
            vec![PreconditionEvaluation::passed(
                "test.claim",
                "operator claims the mutation is safe",
            )]
        }

        fn transform(&self, _sources: &[SourceDocument]) -> Result<Vec<SourceDocument>, String> {
            Ok(source(&two_adds("changed", "also changed")))
        }
    }

    #[test]
    fn operator_cannot_self_certify_a_semantic_mutation() {
        let error = generate_certified_variant(source(PRETTY_SOURCE), &SemanticMutationOperator)
            .unwrap_err();

        assert_eq!(error.code(), "XUNLIE-VARIANT-NOT-EQUIVALENT");
        assert_eq!(error.diagnostics().len(), 1);
        assert_eq!(error.diagnostics()[0].code, "XUNLIE-VARIANT-NOT-EQUIVALENT");
    }

    proptest! {
        #[test]
        fn independent_add_reversal_is_metamorphically_equivalent(
            first in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,39}",
            second in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,39}",
        ) {
            let baseline = source(&two_adds(&first, &second));
            let variant = certified(
                generate_certified_variant(
                    baseline.clone(),
                    &ReverseIndependentAddsOperator,
                )
                .unwrap(),
            );

            prop_assert_eq!(
                &variant.certificate().before().content_digest,
                &variant.certificate().after().content_digest,
            );
            prop_assert!(verify_certified_variant(baseline, &variant).is_ok());
        }
    }
}
