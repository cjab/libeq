use std::collections::HashMap;

use encoding_rs::WINDOWS_1252;

use nom::number::complete::le_i32;
use nom::IResult;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StringReference(i32);

impl StringReference {
    pub fn new(idx: i32) -> Self {
        Self(idx)
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, idx) = le_i32(input)?;
        Ok((remaining, Self::new(idx)))
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

#[derive(Debug)]
pub struct StringHash(HashMap<usize, String>);

const XOR_KEY: [u8; 8] = [0x95, 0x3a, 0xc5, 0x2a, 0x95, 0x7a, 0x95, 0x6a];

pub fn decode_string(encoded_data: &[u8]) -> String {
    let data: Vec<u8> = encoded_data
        .iter()
        .zip(XOR_KEY.iter().cycle())
        .map(|(encoded_char, key_char)| encoded_char ^ key_char)
        .collect();
    let (cow, _, _) = WINDOWS_1252.decode(&data);
    cow.into_owned().trim_end_matches('\u{0}').to_string()
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
        StringHash(
            strings
                .zip(indices)
                .fold(HashMap::new(), |mut hash, (string, (end_idx, _))| {
                    hash.insert(reference_idx, string.to_string());
                    reference_idx = end_idx + 1;
                    hash
                }),
        )
    }

    pub fn serialize(&self) -> Vec<u8> {
        let decoded_string: String = self.0.iter().map(|(_, string)| string.clone()).collect();
        encode_string(&decoded_string)
    }

    pub fn get(&self, string_reference: StringReference) -> Option<&str> {
        self.0
            .get(&(string_reference.0.abs() as usize))
            .map(|s| s.as_ref())
    }
}
