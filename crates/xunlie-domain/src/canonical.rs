use serde::Serialize;
use thiserror::Error;

/// Failure to serialize a typed value into Xunlie canonical JSON.
#[derive(Debug, Error)]
#[error("canonical JSON serialization failed: {0}")]
pub struct CanonicalizationError(#[from] serde_json::Error);

/// Serializes a type whose fields and maps have deterministic order.
///
/// Xunlie canonical JSON v1 is compact UTF-8 JSON. Domain types never contain
/// floating-point values, maps use `BTreeMap`, and struct field declaration
/// order is part of the schema-versioned format.
pub(crate) fn to_canonical_json<T: Serialize>(value: &T) -> Result<String, CanonicalizationError> {
    Ok(serde_json::to_string(value)?)
}
