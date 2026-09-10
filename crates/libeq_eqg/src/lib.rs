//! Identify EverQuest EQG resources and parse binary zone descriptors.
//!
//! [`identify`] inspects headers only; [`zone::parse`] validates binary EQGZ
//! version 1 and 2 descriptors. Archive extraction belongs to `libeq_pfs`.

pub mod zone;

/// A recognized format header, including the raw version for binary resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatHeader {
    /// Binary zone descriptor (`EQGZ`).
    Zone { version: u32 },
    /// Binary terrain mesh (`EQGT`).
    Terrain { version: u32 },
    /// Binary model mesh (`EQGM`).
    Model { version: u32 },
    /// Text terrain project (`EQTZP`); no binary version is read.
    TerrainProject,
    /// Text object group (`EQOBG`); no binary version is read.
    ObjectGroup,
}

/// A recognized header prefix whose remaining bytes are missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("truncated EQG header: expected at least {expected} bytes, got {actual}")]
    TruncatedHeader {
        /// Minimum total length needed for the signature, or eight bytes once a
        /// complete binary signature establishes that a version must follow.
        expected: usize,
        /// Number of input bytes available.
        actual: usize,
    },
}

/// Identify a resource from its signature at byte zero.
///
/// Binary versions are little-endian `u32` values and are returned unchanged,
/// including unknown versions. Text signatures consume no version field.
/// Trailing data is ignored; this function does not validate a complete file.
///
/// Returns `Ok(None)` for empty input or an unknown signature. A nonempty prefix
/// of a known signature, or a binary signature with fewer than four version
/// bytes, returns [`Error::TruncatedHeader`]. Signatures are case-sensitive;
/// whitespace and byte-order marks are not skipped.
///
/// ```
/// use libeq_eqg::{FormatHeader, identify};
/// assert_eq!(identify(b"EQGZ\x02\0\0\0"),
///            Ok(Some(FormatHeader::Zone { version: 2 })));
/// assert_eq!(identify(b"unrecognized"), Ok(None));
/// ```
pub fn identify(input: &[u8]) -> Result<Option<FormatHeader>, Error> {
    if input.is_empty() {
        return Ok(None);
    }

    const SIGNATURES: [&[u8]; 5] = [b"EQGZ", b"EQGT", b"EQGM", b"EQTZP", b"EQOBG"];
    for magic in SIGNATURES {
        if input.len() < magic.len() && magic.starts_with(input) {
            return Err(Error::TruncatedHeader {
                expected: magic.len(),
                actual: input.len(),
            });
        }
        if !input.starts_with(magic) {
            continue;
        }
        if magic == b"EQTZP" {
            return Ok(Some(FormatHeader::TerrainProject));
        }
        if magic == b"EQOBG" {
            return Ok(Some(FormatHeader::ObjectGroup));
        }
        let version_bytes = input.get(4..8).ok_or(Error::TruncatedHeader {
            expected: 8,
            actual: input.len(),
        })?;
        let version = u32::from_le_bytes([
            version_bytes[0],
            version_bytes[1],
            version_bytes[2],
            version_bytes[3],
        ]);
        return Ok(Some(match magic {
            b"EQGZ" => FormatHeader::Zone { version },
            b"EQGT" => FormatHeader::Terrain { version },
            _ => FormatHeader::Model { version },
        }));
    }
    Ok(None)
}

/// Raw terrain and model geometry.
pub mod mesh;
