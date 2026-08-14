use core::fmt;
use serde::{Deserialize, Serialize};

use crate::{Sha256Digest, SourceIdentity};

/// Machine-actionable severity of a diagnostic.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The operation cannot produce a valid artifact.
    Error,
    /// The artifact remains usable but deserves attention.
    Warning,
}

/// A stable location in an ingested source.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// Logical identity supplied by the caller.
    pub identity: SourceIdentity,
    /// Digest of the exact source bytes.
    pub digest: Sha256Digest,
    /// Zero-based position of the source in the explicit history.
    pub source_position: usize,
    /// Zero-based operation position, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_position: Option<usize>,
    /// One-based line, when reported by a parser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// One-based column, when reported by a parser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl SourceLocation {
    /// Creates a location for a source as a whole.
    #[must_use]
    pub fn source(identity: SourceIdentity, digest: Sha256Digest, source_position: usize) -> Self {
        Self {
            identity,
            digest,
            source_position,
            operation_position: None,
            line: None,
            column: None,
        }
    }

    /// Adds an operation position.
    #[must_use]
    pub const fn at_operation(mut self, operation_position: usize) -> Self {
        self.operation_position = Some(operation_position);
        self
    }

    /// Adds a parser line and column.
    #[must_use]
    pub const fn at_text(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

/// A secondary location that explains a diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRelated {
    /// Human-readable relationship to the primary location.
    pub message: String,
    /// Location of the related input.
    pub location: SourceLocation,
}

/// A structured compiler or resolver diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Stable code suitable for CI policy and documentation.
    pub code: String,
    /// Severity that determines artifact usability.
    pub severity: Severity,
    /// Concise human-readable explanation.
    pub message: String,
    /// Primary source location, if the failure came from input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<SourceLocation>,
    /// Other locations needed to understand a conflict.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<DiagnosticRelated>,
}

impl Diagnostic {
    /// Creates an error diagnostic.
    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity: Severity::Error,
            message: message.into(),
            primary: None,
            related: Vec::new(),
        }
    }

    /// Associates a primary location.
    #[must_use]
    pub fn with_primary(mut self, primary: SourceLocation) -> Self {
        self.primary = Some(primary);
        self
    }

    /// Associates a source that contributed to the same failure.
    #[must_use]
    pub fn with_related(mut self, message: impl Into<String>, location: SourceLocation) -> Self {
        self.related.push(DiagnosticRelated {
            message: message.into(),
            location,
        });
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)?;
        if let Some(primary) = &self.primary {
            write!(
                formatter,
                " [{} at source {}, operation {}]",
                primary.identity,
                primary.source_position,
                primary
                    .operation_position
                    .map_or_else(|| "?".to_owned(), |position| position.to_string())
            )?;
        }
        Ok(())
    }
}
