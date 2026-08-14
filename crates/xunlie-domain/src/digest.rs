use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use sha2::{Digest as _, Sha256};
use thiserror::Error;

const PREFIX: &str = "sha256:";
const HEX_LENGTH: usize = 64;

/// A validated, lowercase SHA-256 content digest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    /// Hashes exactly the supplied bytes.
    #[must_use]
    pub fn of_bytes(bytes: impl AsRef<[u8]>) -> Self {
        let hash = Sha256::digest(bytes.as_ref());
        Self(format!("{PREFIX}{hash:x}"))
    }

    /// Parses the `sha256:<64 lowercase hex digits>` representation.
    pub fn parse(value: impl Into<String>) -> Result<Self, DigestParseError> {
        let value = value.into();
        let Some(hex) = value.strip_prefix(PREFIX) else {
            return Err(DigestParseError::MissingPrefix);
        };
        if hex.len() != HEX_LENGTH {
            return Err(DigestParseError::InvalidLength(hex.len()));
        }
        if !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(DigestParseError::InvalidHex);
        }
        Ok(Self(value))
    }

    /// Returns the normalized textual representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Sha256Digest {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for Sha256Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(de::Error::custom)
    }
}

/// Why a digest string was rejected.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DigestParseError {
    /// The algorithm prefix was not `sha256:`.
    #[error("digest must start with `sha256:`")]
    MissingPrefix,
    /// SHA-256 requires exactly 64 hexadecimal digits.
    #[error("digest contains {0} hexadecimal digits; expected 64")]
    InvalidLength(usize),
    /// Uppercase or non-hexadecimal characters were present.
    #[error("digest must contain lowercase hexadecimal digits only")]
    InvalidHex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_known_vector() {
        assert_eq!(
            Sha256Digest::of_bytes(b"abc").as_str(),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn deserialize_rejects_noncanonical_digest() {
        let raw = r#""SHA256:BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD""#;
        assert!(serde_json::from_str::<Sha256Digest>(raw).is_err());
    }
}
