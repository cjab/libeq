use std::collections::BTreeMap;

use super::WResult;
use encoding_rs::WINDOWS_1252;
use nom::number::complete::le_i32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StringReference(pub i32);

impl StringReference {
    pub fn new(idx: i32) -> Self {
        Self(idx)
    }
    pub fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (remaining, idx) = le_i32(input)?;
        Ok((remaining, Self::new(idx)))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct StringHash(BTreeMap<usize, String>);

const XOR_KEY: [u8; 8] = [0x95, 0x3a, 0xc5, 0x2a, 0x95, 0x7a, 0x95, 0x6a];

pub fn decode_string(encoded_data: &[u8]) -> String {
    let data: Vec<u8> = encoded_data
        .iter()
        .zip(XOR_KEY.iter().cycle())
        .map(|(encoded_char, key_char)| encoded_char ^ key_char)
        .collect();
    let (cow, _, _) = WINDOWS_1252.decode(&data);
    cow.into_owned().to_string()
}

pub fn encode_string(decoded_data: &str) -> Vec<u8> {
    let (windows_string, _, _) = WINDOWS_1252.encode(decoded_data);
    windows_string
        .iter()
        .zip(XOR_KEY.iter().cycle())
        .map(|(encoded_char, key_char)| encoded_char ^ key_char)
        .collect()
}

impl StringHash {
    pub fn new(encoded_data: &[u8]) -> StringHash {
        let decoded_string = decode_string(encoded_data);
        let strings = decoded_string.split("\0");
        let indices = decoded_string.match_indices("\0");

        let mut reference_idx = 0;
        StringHash(strings.zip(indices).fold(
            BTreeMap::new(),
            |mut hash, (string, (end_idx, _))| {
                hash.insert(reference_idx, string.to_string());
                reference_idx = end_idx + 1;
                hash
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let decoded_string: String = self.0.values().cloned().map(|s| s + "\0").collect();
        let mut encoded_string = encode_string(&decoded_string);
        let size = encoded_string.len();
        // String data must be padded so that it aligns on 4 bytes
        if !size.is_multiple_of(4) {
            let padding = 4 - (size % 4);
            encoded_string.resize(size + padding, 0);
        }
        encoded_string
    }

    pub fn get(&self, string_reference: StringReference) -> Option<&str> {
        self.0
            .get(&(string_reference.0.unsigned_abs() as usize))
            .map(|s| s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_data() -> Vec<u8> {
        // Build a small string table: "" at offset 0, "FOO" at offset 1, "BAR" at offset 5
        let decoded = "\0FOO\0BAR\0";
        let mut encoded = encode_string(decoded);
        // Pad to 4-byte alignment
        let size = encoded.len();
        if !size.is_multiple_of(4) {
            encoded.resize(size + (4 - size % 4), 0);
        }
        encoded
    }

    #[test]
    fn it_parses() {
        let data = fixture_data();
        let string_hash = StringHash::new(&data);
        assert_eq!(string_hash.get(StringReference::new(0)), Some(""));
        assert_eq!(string_hash.get(StringReference::new(1)), Some("FOO"));
        assert_eq!(string_hash.get(StringReference::new(5)), Some("BAR"));
        assert_eq!(string_hash.get(StringReference::new(99)), None);
    }

    #[test]
    fn it_serializes() {
        let data = fixture_data();
        let string_hash = StringHash::new(&data);
        let serialized = string_hash.to_bytes();
        assert_eq!(data, serialized);
    }
}
