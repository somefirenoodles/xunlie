use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    CanonicalizationError, Diagnostic, Producer, Sha256Digest, canonical::to_canonical_json,
};

/// Schema identifier for persisted equivalence certificates.
pub const EQUIVALENCE_CERTIFICATE_SCHEMA_VERSION: &str = "xunlie.equivalence-certificate/v1";

/// Proof procedure implemented by the M2 engine.
pub const RECOMPILE_PROOF_METHOD: &str = "xunlie.recompile-and-compare/v1";

/// Stable identity of the deterministic operator that proposed a variant.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VariantOperatorIdentity {
    /// Stable operator name. Meaning changes require a new name or major version.
    pub id: String,
    /// Operator implementation version.
    pub version: String,
}

impl VariantOperatorIdentity {
    /// Creates an operator identity.
    #[must_use]
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
        }
    }

    fn validate(&self, diagnostics: &mut Vec<Diagnostic>) {
        if self.id.trim().is_empty() || self.version.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-OPERATOR",
                "operator id and version must not be empty",
            ));
        }
    }
}

/// Result of evaluating one executable operator precondition.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PreconditionStatus {
    /// The input satisfies the precondition.
    Passed,
    /// The input does not satisfy the precondition and must be excluded.
    Failed,
}

/// Auditable evidence for one executable precondition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PreconditionEvaluation {
    /// Stable check identifier.
    pub id: String,
    /// Machine-readable result.
    pub status: PreconditionStatus,
    /// Human-readable observation explaining the result.
    pub explanation: String,
}

impl PreconditionEvaluation {
    /// Records a satisfied precondition.
    #[must_use]
    pub fn passed(id: impl Into<String>, explanation: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: PreconditionStatus::Passed,
            explanation: explanation.into(),
        }
    }

    /// Records a failed precondition and its exclusion reason.
    #[must_use]
    pub fn failed(id: impl Into<String>, explanation: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: PreconditionStatus::Failed,
            explanation: explanation.into(),
        }
    }

    /// Returns whether the precondition passed.
    #[must_use]
    pub const fn is_passed(&self) -> bool {
        matches!(self.status, PreconditionStatus::Passed)
    }

    fn validate(&self, diagnostics: &mut Vec<Diagnostic>) {
        if self.id.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-PRECONDITION-ID",
                "precondition id must not be empty",
            ));
        }
        if self.explanation.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-PRECONDITION-EXPLANATION",
                format!("precondition `{}` must include an explanation", self.id),
            ));
        }
    }
}

/// Digests that bind one side of a certified transformation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CertifiedHistory {
    /// Digest over ordered source identities, positions, and exact source bytes.
    pub history_digest: Sha256Digest,
    /// Semantic digest of the compiled effective contract.
    pub content_digest: Sha256Digest,
    /// Integrity digest of the compiled ContractIR including provenance.
    pub artifact_digest: Sha256Digest,
}

/// Evidence that two different histories compile to one effective contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EquivalenceProof {
    /// Versioned proof procedure.
    pub method: String,
    /// Result recorded only after executing the proof procedure.
    pub result: EquivalenceProofResult,
}

/// Closed result set for an equivalence proof.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EquivalenceProofResult {
    /// Both histories compiled successfully and their semantic digests matched.
    Equivalent,
}

/// Deterministic, self-validating certificate for one history variant.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EquivalenceCertificate {
    schema_version: String,
    producer: Producer,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    operator: VariantOperatorIdentity,
    preconditions: Vec<PreconditionEvaluation>,
    before: CertifiedHistory,
    after: CertifiedHistory,
    proof: EquivalenceProof,
    #[serde(rename = "contentDigest")]
    certificate_digest: Sha256Digest,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CertificatePayload<'a> {
    schema_version: &'a str,
    producer: &'a Producer,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<&'a str>,
    operator: &'a VariantOperatorIdentity,
    preconditions: &'a [PreconditionEvaluation],
    before: &'a CertifiedHistory,
    after: &'a CertifiedHistory,
    proof: &'a EquivalenceProof,
}

impl EquivalenceCertificate {
    /// Creates a certificate after all preconditions and equivalence checks ran.
    pub fn new(
        producer: Producer,
        operator: VariantOperatorIdentity,
        preconditions: Vec<PreconditionEvaluation>,
        before: CertifiedHistory,
        after: CertifiedHistory,
    ) -> Result<Self, Vec<Diagnostic>> {
        let proof = EquivalenceProof {
            method: RECOMPILE_PROOF_METHOD.to_owned(),
            result: EquivalenceProofResult::Equivalent,
        };
        let certificate_digest = certificate_digest(
            EQUIVALENCE_CERTIFICATE_SCHEMA_VERSION,
            &producer,
            None,
            &operator,
            &preconditions,
            &before,
            &after,
            &proof,
        )
        .map_err(|error| {
            vec![Diagnostic::error(
                "XUNLIE-CANONICAL-SERIALIZE",
                error.to_string(),
            )]
        })?;
        let certificate = Self {
            schema_version: EQUIVALENCE_CERTIFICATE_SCHEMA_VERSION.to_owned(),
            producer,
            created_at: None,
            operator,
            preconditions,
            before,
            after,
            proof,
            certificate_digest,
        };
        certificate.validate()?;
        Ok(certificate)
    }

    /// Validates the schema, proof evidence, equivalence, and certificate digest.
    pub fn validate(&self) -> Result<(), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != EQUIVALENCE_CERTIFICATE_SCHEMA_VERSION {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-SCHEMA",
                format!(
                    "unsupported certificate schema `{}`; expected `{EQUIVALENCE_CERTIFICATE_SCHEMA_VERSION}`",
                    self.schema_version
                ),
            ));
        }
        if self.producer.name.trim().is_empty() || self.producer.version.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-PRODUCER",
                "certificate producer name and version must not be empty",
            ));
        }
        self.operator.validate(&mut diagnostics);

        if self.preconditions.is_empty() {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-NO-PRECONDITIONS",
                "a certificate must contain executable precondition evidence",
            ));
        }
        let mut precondition_ids = BTreeSet::new();
        for precondition in &self.preconditions {
            precondition.validate(&mut diagnostics);
            if !precondition_ids.insert(precondition.id.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "XUNLIE-CERTIFICATE-DUPLICATE-PRECONDITION",
                    format!("precondition `{}` appears more than once", precondition.id),
                ));
            }
            if !precondition.is_passed() {
                diagnostics.push(Diagnostic::error(
                    "XUNLIE-CERTIFICATE-FAILED-PRECONDITION",
                    format!(
                        "precondition `{}` failed and cannot appear in a certificate",
                        precondition.id
                    ),
                ));
            }
        }

        if self.proof.method != RECOMPILE_PROOF_METHOD
            || self.proof.result != EquivalenceProofResult::Equivalent
        {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-PROOF",
                format!("proof must record `{RECOMPILE_PROOF_METHOD}` with an equivalent result"),
            ));
        }
        if self.before.content_digest != self.after.content_digest {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-NOT-EQUIVALENT",
                format!(
                    "before content digest `{}` differs from after `{}`",
                    self.before.content_digest, self.after.content_digest
                ),
            ));
        }
        if self.before.history_digest == self.after.history_digest {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-UNCHANGED-HISTORY",
                "a variant must change the exact history representation",
            ));
        }
        if self.before.artifact_digest == self.after.artifact_digest {
            diagnostics.push(Diagnostic::error(
                "XUNLIE-CERTIFICATE-UNCHANGED-ARTIFACT",
                "a changed source history must change ContractIR provenance",
            ));
        }

        match certificate_digest(
            &self.schema_version,
            &self.producer,
            self.created_at.as_deref(),
            &self.operator,
            &self.preconditions,
            &self.before,
            &self.after,
            &self.proof,
        ) {
            Ok(expected) if expected != self.certificate_digest => {
                diagnostics.push(Diagnostic::error(
                    "XUNLIE-CERTIFICATE-DIGEST-MISMATCH",
                    format!(
                        "certificate digest is `{}` but canonical evidence hashes to `{expected}`",
                        self.certificate_digest
                    ),
                ));
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

    /// Returns compact deterministic JSON for persistence and golden vectors.
    pub fn canonical_json(&self) -> Result<String, CanonicalizationError> {
        to_canonical_json(self)
    }

    /// Returns the operator that produced the variant.
    #[must_use]
    pub const fn operator(&self) -> &VariantOperatorIdentity {
        &self.operator
    }

    /// Returns all successful executable preconditions in evaluation order.
    #[must_use]
    pub fn preconditions(&self) -> &[PreconditionEvaluation] {
        &self.preconditions
    }

    /// Returns the digests bound to the baseline history.
    #[must_use]
    pub const fn before(&self) -> &CertifiedHistory {
        &self.before
    }

    /// Returns the digests bound to the transformed history.
    #[must_use]
    pub const fn after(&self) -> &CertifiedHistory {
        &self.after
    }

    /// Returns the digest over all certificate fields except itself.
    #[must_use]
    pub const fn certificate_digest(&self) -> &Sha256Digest {
        &self.certificate_digest
    }

    /// Returns the proof procedure and result.
    #[must_use]
    pub const fn proof(&self) -> &EquivalenceProof {
        &self.proof
    }

    /// Returns the schema identifier.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

#[allow(clippy::too_many_arguments)]
fn certificate_digest(
    schema_version: &str,
    producer: &Producer,
    created_at: Option<&str>,
    operator: &VariantOperatorIdentity,
    preconditions: &[PreconditionEvaluation],
    before: &CertifiedHistory,
    after: &CertifiedHistory,
    proof: &EquivalenceProof,
) -> Result<Sha256Digest, CanonicalizationError> {
    Ok(Sha256Digest::of_bytes(to_canonical_json(
        &CertificatePayload {
            schema_version,
            producer,
            created_at,
            operator,
            preconditions,
            before,
            after,
            proof,
        },
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: &str) -> Sha256Digest {
        Sha256Digest::of_bytes(value)
    }

    fn history(history: &str, content: &str, artifact: &str) -> CertifiedHistory {
        CertifiedHistory {
            history_digest: digest(history),
            content_digest: digest(content),
            artifact_digest: digest(artifact),
        }
    }

    fn certificate() -> EquivalenceCertificate {
        EquivalenceCertificate::new(
            Producer {
                name: "xunlie-engine".to_owned(),
                version: "0.1.0".to_owned(),
            },
            VariantOperatorIdentity::new("json.presentation.normalize", "1.0.0"),
            vec![PreconditionEvaluation::passed(
                "source.valid-json",
                "all source documents parsed as JSON",
            )],
            history("before-history", "same-contract", "before-artifact"),
            history("after-history", "same-contract", "after-artifact"),
        )
        .unwrap()
    }

    #[test]
    fn valid_certificate_survives_canonical_round_trip() {
        let original = certificate();
        let json = original.canonical_json().unwrap();
        let decoded: EquivalenceCertificate = serde_json::from_str(&json).unwrap();

        decoded.validate().unwrap();
        assert_eq!(decoded, original);
        assert_eq!(decoded.canonical_json().unwrap(), json);
    }

    #[test]
    fn failed_precondition_cannot_be_certified() {
        let result = EquivalenceCertificate::new(
            Producer {
                name: "xunlie-engine".to_owned(),
                version: "0.1.0".to_owned(),
            },
            VariantOperatorIdentity::new("test.operator", "1.0.0"),
            vec![PreconditionEvaluation::failed(
                "test.precondition",
                "input is outside the safe domain",
            )],
            history("before-history", "same-contract", "before-artifact"),
            history("after-history", "same-contract", "after-artifact"),
        );

        assert!(
            result
                .unwrap_err()
                .iter()
                .any(|diagnostic| { diagnostic.code == "XUNLIE-CERTIFICATE-FAILED-PRECONDITION" })
        );
    }

    #[test]
    fn differing_contract_digests_are_rejected() {
        let result = EquivalenceCertificate::new(
            Producer {
                name: "xunlie-engine".to_owned(),
                version: "0.1.0".to_owned(),
            },
            VariantOperatorIdentity::new("test.operator", "1.0.0"),
            vec![PreconditionEvaluation::passed(
                "test.precondition",
                "passed",
            )],
            history("before-history", "before-contract", "before-artifact"),
            history("after-history", "after-contract", "after-artifact"),
        );

        assert!(
            result
                .unwrap_err()
                .iter()
                .any(|diagnostic| { diagnostic.code == "XUNLIE-CERTIFICATE-NOT-EQUIVALENT" })
        );
    }

    #[test]
    fn tampering_invalidates_certificate_digest() {
        let original = certificate();
        let mut value: serde_json::Value =
            serde_json::from_str(&original.canonical_json().unwrap()).unwrap();
        value["preconditions"][0]["explanation"] =
            serde_json::Value::String("tampered evidence".to_owned());
        let tampered: EquivalenceCertificate = serde_json::from_value(value).unwrap();

        assert!(
            tampered
                .validate()
                .unwrap_err()
                .iter()
                .any(|diagnostic| { diagnostic.code == "XUNLIE-CERTIFICATE-DIGEST-MISMATCH" })
        );
    }

    #[test]
    fn operator_id_and_version_are_independently_required() {
        for (id, version) in [("", "1.0.0"), ("test.operator", "")] {
            let result = EquivalenceCertificate::new(
                Producer {
                    name: "xunlie-engine".to_owned(),
                    version: "0.1.0".to_owned(),
                },
                VariantOperatorIdentity::new(id, version),
                vec![PreconditionEvaluation::passed(
                    "test.precondition",
                    "passed",
                )],
                history("before-history", "same-contract", "before-artifact"),
                history("after-history", "same-contract", "after-artifact"),
            );

            assert!(
                result
                    .unwrap_err()
                    .iter()
                    .any(|diagnostic| diagnostic.code == "XUNLIE-CERTIFICATE-OPERATOR")
            );
        }
    }

    #[test]
    fn producer_name_and_version_are_independently_required() {
        for (name, version) in [("", "0.1.0"), ("xunlie-engine", "")] {
            let result = EquivalenceCertificate::new(
                Producer {
                    name: name.to_owned(),
                    version: version.to_owned(),
                },
                VariantOperatorIdentity::new("test.operator", "1.0.0"),
                vec![PreconditionEvaluation::passed(
                    "test.precondition",
                    "passed",
                )],
                history("before-history", "same-contract", "before-artifact"),
                history("after-history", "same-contract", "after-artifact"),
            );

            assert!(
                result
                    .unwrap_err()
                    .iter()
                    .any(|diagnostic| diagnostic.code == "XUNLIE-CERTIFICATE-PRODUCER")
            );
        }
    }

    #[test]
    fn precondition_id_and_explanation_are_required() {
        for precondition in [
            PreconditionEvaluation::passed("", "passed"),
            PreconditionEvaluation::passed("test.precondition", ""),
        ] {
            let result = EquivalenceCertificate::new(
                Producer {
                    name: "xunlie-engine".to_owned(),
                    version: "0.1.0".to_owned(),
                },
                VariantOperatorIdentity::new("test.operator", "1.0.0"),
                vec![precondition],
                history("before-history", "same-contract", "before-artifact"),
                history("after-history", "same-contract", "after-artifact"),
            );

            assert!(result.is_err());
        }
    }

    #[test]
    fn proof_method_is_validated_independently_from_result() {
        let original = certificate();
        let mut value: serde_json::Value =
            serde_json::from_str(&original.canonical_json().unwrap()).unwrap();
        value["proof"]["method"] = serde_json::Value::String("test.forged/v1".to_owned());
        let tampered: EquivalenceCertificate = serde_json::from_value(value).unwrap();
        let diagnostics = tampered.validate().unwrap_err();

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "XUNLIE-CERTIFICATE-PROOF")
        );
    }

    #[test]
    fn public_certificate_evidence_accessors_return_persisted_values() {
        let certificate = certificate();

        assert_eq!(
            certificate.schema_version(),
            EQUIVALENCE_CERTIFICATE_SCHEMA_VERSION
        );
        assert_eq!(certificate.preconditions().len(), 1);
        assert_eq!(certificate.preconditions()[0].id, "source.valid-json");
    }
}
