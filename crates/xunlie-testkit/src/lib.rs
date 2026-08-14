#![forbid(unsafe_code)]

//! Shared fixtures for Xunlie integration and conformance tests.
//!
//! This crate is intentionally unsuitable for production dependencies. It owns
//! disposable workspaces and fixture data, but no product behavior.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use tempfile::TempDir;

/// Smallest representative source accepted by the v1 compiler.
pub const MINIMAL_SOURCE_JSON: &str = include_str!("../fixtures/minimal-source.json");

/// Syntactically valid JSON that is not a supported Xunlie source.
pub const INVALID_SOURCE_JSON: &str = include_str!("../fixtures/invalid-source.json");

/// Two commutative additions used by certified history-variant tests.
pub const INDEPENDENT_ADDS_SOURCE_JSON: &str =
    include_str!("../fixtures/independent-adds-source.json");

/// A disposable filesystem root with convenience methods for arranging tests.
#[derive(Debug)]
pub struct FixtureWorkspace {
    root: TempDir,
}

impl FixtureWorkspace {
    /// Creates a disposable workspace.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error if a temporary directory cannot be
    /// created.
    pub fn new() -> io::Result<Self> {
        tempfile::tempdir().map(|root| Self { root })
    }

    /// Returns the temporary workspace root.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.root.path()
    }

    /// Writes a fixture below the workspace root, creating parent directories.
    ///
    /// # Errors
    ///
    /// Returns the underlying filesystem error from directory creation or the
    /// file write.
    pub fn write(
        &self,
        relative: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> io::Result<PathBuf> {
        let path = self.root.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents)?;
        Ok(path)
    }

    /// Reads a fixture below the workspace root.
    ///
    /// # Errors
    ///
    /// Returns the underlying filesystem error when the file cannot be read.
    pub fn read(&self, relative: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        fs::read(self.root.path().join(relative))
    }

    /// Writes the canonical minimal source fixture to `source.json`.
    ///
    /// # Errors
    ///
    /// Returns the underlying filesystem error from the write.
    pub fn write_minimal_source(&self) -> io::Result<PathBuf> {
        self.write("source.json", MINIMAL_SOURCE_JSON)
    }
}

/// Assert that a JSON document has the expected root schema identifier.
///
/// # Panics
///
/// Panics when `document` is not JSON or its `schemaVersion` differs from
/// `expected`.
pub fn assert_schema_version(document: &[u8], expected: &str) {
    let value: serde_json::Value =
        serde_json::from_slice(document).expect("fixture must be valid JSON");
    assert_eq!(
        value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_str),
        Some(expected),
        "unexpected or missing schemaVersion"
    );
}

/// Cross-platform path identity suitable for source provenance in tests.
#[must_use]
pub fn fixture_identity(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_writes_nested_fixture() {
        let workspace = FixtureWorkspace::new().unwrap();
        let path = workspace.write("nested/source.json", b"{}\n").unwrap();

        assert!(path.starts_with(workspace.path()));
        assert_eq!(workspace.read("nested/source.json").unwrap(), b"{}\n");
    }

    #[test]
    fn schema_assertion_accepts_expected_version() {
        assert_schema_version(
            br#"{"schemaVersion":"xunlie.contract/v1"}"#,
            "xunlie.contract/v1",
        );
    }

    #[test]
    fn identity_normalizes_windows_separators() {
        assert_eq!(
            fixture_identity(Path::new(r"fixture\source.json")),
            "fixture/source.json"
        );
    }

    #[test]
    fn embedded_sources_are_json() {
        serde_json::from_str::<serde_json::Value>(MINIMAL_SOURCE_JSON).unwrap();
        serde_json::from_str::<serde_json::Value>(INVALID_SOURCE_JSON).unwrap();
        serde_json::from_str::<serde_json::Value>(INDEPENDENT_ADDS_SOURCE_JSON).unwrap();
    }
}
