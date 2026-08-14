use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Diagnostic, Requirement, RequirementId, SourceLocation, SourceRecord};

/// Versioned policy controlling how ambiguous histories are handled.
///
/// M1 intentionally supports only strict rejection. Adding a policy that
/// guesses intent requires a schema change and explicit design review.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionPolicy {
    /// Contradictory adds and invalid targets make compilation fail closed.
    #[default]
    Strict,
}

/// Total, explicit ordering key for a history operation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Precedence {
    /// Position of the owning source.
    pub source_position: usize,
    /// Position of the operation inside that source.
    pub operation_position: usize,
}

/// A history mutation before resolution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "op")]
pub enum Operation {
    /// Introduces a requirement that does not contradict an active definition.
    Add {
        /// Requirement to introduce.
        requirement: Requirement,
    },
    /// Supersedes the active definition of the target.
    Replace {
        /// Existing requirement identifier.
        target: RequirementId,
        /// New definition; its embedded ID must equal `target`.
        requirement: Requirement,
    },
    /// Removes the active definition of the target.
    Revoke {
        /// Existing requirement identifier.
        target: RequirementId,
    },
}

impl Operation {
    fn target(&self) -> &RequirementId {
        match self {
            Self::Add { requirement } => requirement.id(),
            Self::Replace { target, .. } | Self::Revoke { target } => target,
        }
    }
}

/// One located operation in a history.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct HistoryEvent {
    /// Ordering key, independent from vector/storage order.
    pub precedence: Precedence,
    /// Source location used in conflict diagnostics.
    pub location: SourceLocation,
    /// Mutation to apply.
    pub operation: Operation,
}

/// A complete history submitted to the pure resolver.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct History {
    /// Explicit conflict policy.
    pub policy: ResolutionPolicy,
    /// Source provenance referenced by every event.
    pub sources: Vec<SourceRecord>,
    /// Events; vector order is irrelevant because precedence is explicit.
    pub events: Vec<HistoryEvent>,
}

/// Mutation actually applied by the resolver.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionAction {
    /// A requirement became active.
    Added,
    /// An active requirement was superseded.
    Replaced,
    /// An active requirement was revoked.
    Revoked,
    /// An identical add or replacement did not change semantic content.
    NoChange,
}

/// Auditable result for one successful event.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResolutionDecision {
    /// Ordering key of the applied event.
    pub precedence: Precedence,
    /// Requirement affected by the event.
    pub target: RequirementId,
    /// State transition performed.
    pub action: ResolutionAction,
    /// Source that authorized the transition.
    pub location: SourceLocation,
}

/// Effective contract and its successful resolution trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedContract {
    requirements: BTreeMap<RequirementId, Requirement>,
    decisions: Vec<ResolutionDecision>,
}

impl ResolvedContract {
    /// Returns effective requirements in stable identifier order.
    #[must_use]
    pub const fn requirements(&self) -> &BTreeMap<RequirementId, Requirement> {
        &self.requirements
    }

    /// Consumes the result and returns its effective requirements.
    #[must_use]
    pub fn into_requirements(self) -> BTreeMap<RequirementId, Requirement> {
        self.requirements
    }

    /// Returns the ordered resolution trace.
    #[must_use]
    pub fn decisions(&self) -> &[ResolutionDecision] {
        &self.decisions
    }
}

/// Stable classification of an unresolved contradiction.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    /// Two sources claim the same identity or source position.
    DuplicateSource,
    /// Two events claim the same total ordering key.
    DuplicatePrecedence,
    /// An event's location does not match its declared source.
    UnknownSource,
    /// A second add differs from the active definition.
    ContradictoryAdd,
    /// A replacement names a target that is not active.
    MissingReplacementTarget,
    /// A replacement attempts an implicit rename.
    ReplacementIdMismatch,
    /// A revocation names a target that is not active.
    MissingRevocationTarget,
    /// A deserialized requirement violates a semantic invariant.
    InvalidRequirement,
}

/// One conflict plus its structured diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Conflict {
    /// Machine-actionable kind.
    pub kind: ConflictKind,
    /// Requirement affected, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<RequirementId>,
    /// Full source-aware explanation.
    pub diagnostic: Diagnostic,
}

/// Failure returned instead of a partially resolved contract.
#[derive(Clone, Debug, Error)]
#[error("history resolution failed with {} conflict(s)", .conflicts.len())]
pub struct ResolutionFailure {
    conflicts: Vec<Conflict>,
}

impl ResolutionFailure {
    /// Returns all deterministic conflicts. No partial contract is retained.
    #[must_use]
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Returns cloned structured diagnostics for compiler aggregation.
    #[must_use]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.conflicts
            .iter()
            .map(|conflict| conflict.diagnostic.clone())
            .collect()
    }
}

#[derive(Clone)]
struct ActiveRequirement {
    requirement: Requirement,
    origin: SourceLocation,
}

/// Resolves a history deterministically, or returns conflicts without partial IR.
pub fn resolve_history(history: &History) -> Result<ResolvedContract, ResolutionFailure> {
    let mut conflicts = validate_history_structure(history);
    if !conflicts.is_empty() {
        return Err(ResolutionFailure { conflicts });
    }
    let mut events: Vec<&HistoryEvent> = history.events.iter().collect();
    events.sort_by_key(|event| event.precedence);

    let mut active: BTreeMap<RequirementId, ActiveRequirement> = BTreeMap::new();
    let mut decisions = Vec::with_capacity(events.len());

    for event in events {
        let target = event.operation.target().clone();
        match &event.operation {
            Operation::Add { requirement } => {
                if let Err(diagnostic) = requirement.validate() {
                    let mut diagnostic = *diagnostic;
                    diagnostic.primary = Some(event.location.clone());
                    conflicts.push(Conflict {
                        kind: ConflictKind::InvalidRequirement,
                        target: Some(target),
                        diagnostic,
                    });
                    continue;
                }
                match active.get(requirement.id()) {
                    None => {
                        active.insert(
                            requirement.id().clone(),
                            ActiveRequirement {
                                requirement: requirement.clone(),
                                origin: event.location.clone(),
                            },
                        );
                        decisions.push(decision(event, target, ResolutionAction::Added));
                    }
                    Some(previous) if previous.requirement == *requirement => {
                        decisions.push(decision(event, target, ResolutionAction::NoChange));
                    }
                    Some(previous) => conflicts.push(Conflict {
                        kind: ConflictKind::ContradictoryAdd,
                        target: Some(target.clone()),
                        diagnostic: Diagnostic::error(
                            "XUNLIE-RESOLVE-CONTRADICTORY-ADD",
                            format!(
                                "add for `{target}` contradicts its active definition; use replace explicitly"
                            ),
                        )
                        .with_primary(event.location.clone())
                        .with_related("active definition was introduced here", previous.origin.clone()),
                    }),
                }
            }
            Operation::Replace {
                target,
                requirement,
            } => {
                if requirement.id() != target {
                    conflicts.push(Conflict {
                        kind: ConflictKind::ReplacementIdMismatch,
                        target: Some(target.clone()),
                        diagnostic: Diagnostic::error(
                            "XUNLIE-RESOLVE-REPLACE-ID-MISMATCH",
                            format!(
                                "replace target `{target}` does not match replacement id `{}`",
                                requirement.id()
                            ),
                        )
                        .with_primary(event.location.clone()),
                    });
                    continue;
                }
                if let Err(diagnostic) = requirement.validate() {
                    let mut diagnostic = *diagnostic;
                    diagnostic.primary = Some(event.location.clone());
                    conflicts.push(Conflict {
                        kind: ConflictKind::InvalidRequirement,
                        target: Some(target.clone()),
                        diagnostic,
                    });
                    continue;
                }
                match active.get_mut(target) {
                    None => conflicts.push(Conflict {
                        kind: ConflictKind::MissingReplacementTarget,
                        target: Some(target.clone()),
                        diagnostic: Diagnostic::error(
                            "XUNLIE-RESOLVE-MISSING-REPLACE-TARGET",
                            format!("cannot replace `{target}` because it is not active"),
                        )
                        .with_primary(event.location.clone()),
                    }),
                    Some(previous) => {
                        let action = if previous.requirement == *requirement {
                            ResolutionAction::NoChange
                        } else {
                            previous.requirement = requirement.clone();
                            previous.origin = event.location.clone();
                            ResolutionAction::Replaced
                        };
                        decisions.push(decision(event, target.clone(), action));
                    }
                }
            }
            Operation::Revoke { target } => {
                if active.remove(target).is_some() {
                    decisions.push(decision(event, target.clone(), ResolutionAction::Revoked));
                } else {
                    conflicts.push(Conflict {
                        kind: ConflictKind::MissingRevocationTarget,
                        target: Some(target.clone()),
                        diagnostic: Diagnostic::error(
                            "XUNLIE-RESOLVE-MISSING-REVOKE-TARGET",
                            format!("cannot revoke `{target}` because it is not active"),
                        )
                        .with_primary(event.location.clone()),
                    });
                }
            }
        }
    }

    if conflicts.is_empty() {
        Ok(ResolvedContract {
            requirements: active
                .into_iter()
                .map(|(id, entry)| (id, entry.requirement))
                .collect(),
            decisions,
        })
    } else {
        Err(ResolutionFailure { conflicts })
    }
}

fn decision(
    event: &HistoryEvent,
    target: RequirementId,
    action: ResolutionAction,
) -> ResolutionDecision {
    ResolutionDecision {
        precedence: event.precedence,
        target,
        action,
        location: event.location.clone(),
    }
}

fn validate_history_structure(history: &History) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    let mut identities: BTreeMap<_, &SourceRecord> = BTreeMap::new();
    let mut positions: BTreeMap<_, &SourceRecord> = BTreeMap::new();
    for source in &history.sources {
        if let Some(previous) = identities.insert(source.identity.clone(), source) {
            conflicts.push(Conflict {
                kind: ConflictKind::DuplicateSource,
                target: None,
                diagnostic: Diagnostic::error(
                    "XUNLIE-RESOLVE-DUPLICATE-SOURCE-IDENTITY",
                    format!(
                        "source identity `{}` appears more than once",
                        source.identity
                    ),
                )
                .with_primary(source.location(0))
                .with_related("first declaration was here", previous.location(0)),
            });
        }
        if let Some(previous) = positions.insert(source.position, source) {
            conflicts.push(Conflict {
                kind: ConflictKind::DuplicateSource,
                target: None,
                diagnostic: Diagnostic::error(
                    "XUNLIE-RESOLVE-DUPLICATE-SOURCE-POSITION",
                    format!("source position {} appears more than once", source.position),
                )
                .with_primary(source.location(0))
                .with_related("first declaration was here", previous.location(0)),
            });
        }
    }

    let source_keys: BTreeSet<_> = history
        .sources
        .iter()
        .map(|source| {
            (
                source.position,
                source.identity.clone(),
                source.digest.clone(),
            )
        })
        .collect();
    let mut precedences: BTreeMap<Precedence, &HistoryEvent> = BTreeMap::new();
    for event in &history.events {
        let source_key = (
            event.location.source_position,
            event.location.identity.clone(),
            event.location.digest.clone(),
        );
        if !source_keys.contains(&source_key)
            || event.precedence.source_position != event.location.source_position
            || event.precedence.operation_position
                != event.location.operation_position.unwrap_or(usize::MAX)
        {
            conflicts.push(Conflict {
                kind: ConflictKind::UnknownSource,
                target: Some(event.operation.target().clone()),
                diagnostic: Diagnostic::error(
                    "XUNLIE-RESOLVE-UNKNOWN-SOURCE",
                    "event location or precedence does not match a declared source",
                )
                .with_primary(event.location.clone()),
            });
        }
        if let Some(previous) = precedences.insert(event.precedence, event) {
            conflicts.push(Conflict {
                kind: ConflictKind::DuplicatePrecedence,
                target: Some(event.operation.target().clone()),
                diagnostic: Diagnostic::error(
                    "XUNLIE-RESOLVE-DUPLICATE-PRECEDENCE",
                    format!(
                        "more than one event claims precedence ({}, {})",
                        event.precedence.source_position, event.precedence.operation_position
                    ),
                )
                .with_primary(event.location.clone())
                .with_related("other event is here", previous.location.clone()),
            });
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::{RequirementKind, RequirementPriority, Severity, Sha256Digest, SourceIdentity};

    fn requirement(id: &str, statement: &str) -> Requirement {
        Requirement::new(
            RequirementId::new(id).unwrap(),
            RequirementKind::Functional,
            RequirementPriority::Must,
            statement,
        )
        .unwrap()
    }

    fn source(position: usize) -> SourceRecord {
        SourceRecord {
            identity: SourceIdentity::new(format!("memory://source-{position}")).unwrap(),
            digest: Sha256Digest::of_bytes(format!("source-{position}")),
            position,
        }
    }

    fn event(
        source: &SourceRecord,
        operation_position: usize,
        operation: Operation,
    ) -> HistoryEvent {
        HistoryEvent {
            precedence: Precedence {
                source_position: source.position,
                operation_position,
            },
            location: source.location(operation_position),
            operation,
        }
    }

    #[test]
    fn add_replace_and_revoke_produce_expected_contract() {
        let source = source(0);
        let history = History {
            policy: ResolutionPolicy::Strict,
            sources: vec![source.clone()],
            events: vec![
                event(
                    &source,
                    0,
                    Operation::Add {
                        requirement: requirement("REQ-1", "first"),
                    },
                ),
                event(
                    &source,
                    1,
                    Operation::Replace {
                        target: RequirementId::new("REQ-1").unwrap(),
                        requirement: requirement("REQ-1", "second"),
                    },
                ),
                event(
                    &source,
                    2,
                    Operation::Add {
                        requirement: requirement("REQ-2", "temporary"),
                    },
                ),
                event(
                    &source,
                    3,
                    Operation::Revoke {
                        target: RequirementId::new("REQ-2").unwrap(),
                    },
                ),
            ],
        };

        let resolved = resolve_history(&history).unwrap();
        assert_eq!(resolved.requirements().len(), 1);
        assert_eq!(
            resolved.requirements()[&RequirementId::new("REQ-1").unwrap()].statement(),
            "second"
        );
        assert_eq!(
            resolved
                .decisions()
                .iter()
                .map(|decision| decision.action)
                .collect::<Vec<_>>(),
            vec![
                ResolutionAction::Added,
                ResolutionAction::Replaced,
                ResolutionAction::Added,
                ResolutionAction::Revoked,
            ]
        );
    }

    #[test]
    fn contradictory_add_reports_both_sources() {
        let first = source(0);
        let second = source(1);
        let history = History {
            policy: ResolutionPolicy::Strict,
            sources: vec![first.clone(), second.clone()],
            events: vec![
                event(
                    &first,
                    0,
                    Operation::Add {
                        requirement: requirement("REQ-1", "first"),
                    },
                ),
                event(
                    &second,
                    0,
                    Operation::Add {
                        requirement: requirement("REQ-1", "different"),
                    },
                ),
            ],
        };

        let failure = resolve_history(&history).unwrap_err();
        assert_eq!(failure.conflicts().len(), 1);
        let conflict = &failure.conflicts()[0];
        assert_eq!(conflict.kind, ConflictKind::ContradictoryAdd);
        assert_eq!(conflict.diagnostic.severity, Severity::Error);
        assert_eq!(conflict.diagnostic.related.len(), 1);
    }

    #[test]
    fn missing_replace_and_revoke_targets_fail_closed() {
        let source = source(0);
        let target = RequirementId::new("REQ-MISSING").unwrap();
        let history = History {
            policy: ResolutionPolicy::Strict,
            sources: vec![source.clone()],
            events: vec![
                event(
                    &source,
                    0,
                    Operation::Replace {
                        target: target.clone(),
                        requirement: requirement("REQ-MISSING", "replacement"),
                    },
                ),
                event(&source, 1, Operation::Revoke { target }),
            ],
        };

        let failure = resolve_history(&history).unwrap_err();
        assert_eq!(failure.conflicts().len(), 2);
        assert_eq!(
            failure.conflicts()[0].kind,
            ConflictKind::MissingReplacementTarget
        );
        assert_eq!(
            failure.conflicts()[1].kind,
            ConflictKind::MissingRevocationTarget
        );
    }

    #[test]
    fn replace_operation_targets_declared_target_not_replacement_id() {
        let target = RequirementId::new("REQ-TARGET").unwrap();
        let operation = Operation::Replace {
            target: target.clone(),
            requirement: requirement("REQ-OTHER", "invalid implicit rename"),
        };

        assert_eq!(operation.target(), &target);
    }

    #[test]
    fn structural_conflict_aborts_before_semantic_resolution() {
        let source = source(0);
        let location = source.location(0);
        let precedence = Precedence {
            source_position: 0,
            operation_position: 0,
        };
        let history = History {
            policy: ResolutionPolicy::Strict,
            sources: vec![source],
            events: vec![
                HistoryEvent {
                    precedence,
                    location: location.clone(),
                    operation: Operation::Add {
                        requirement: requirement("REQ-1", "first"),
                    },
                },
                HistoryEvent {
                    precedence,
                    location,
                    operation: Operation::Add {
                        requirement: requirement("REQ-1", "contradictory"),
                    },
                },
            ],
        };

        let failure = resolve_history(&history).unwrap_err();
        assert_eq!(failure.conflicts().len(), 1);
        assert_eq!(
            failure.conflicts()[0].kind,
            ConflictKind::DuplicatePrecedence
        );
    }

    proptest! {
        #[test]
        fn storage_order_does_not_change_resolution(first in "[a-z]{1,16}", second in "[a-z]{1,16}") {
            let source = source(0);
            let add = event(
                &source,
                0,
                Operation::Add { requirement: requirement("REQ-1", &first) },
            );
            let replace = event(
                &source,
                1,
                Operation::Replace {
                    target: RequirementId::new("REQ-1").unwrap(),
                    requirement: requirement("REQ-1", &second),
                },
            );
            let forward = History {
                policy: ResolutionPolicy::Strict,
                sources: vec![source.clone()],
                events: vec![add.clone(), replace.clone()],
            };
            let reverse = History {
                policy: ResolutionPolicy::Strict,
                sources: vec![source],
                events: vec![replace, add],
            };

            prop_assert_eq!(
                resolve_history(&forward).unwrap().requirements,
                resolve_history(&reverse).unwrap().requirements,
            );
        }
    }
}
