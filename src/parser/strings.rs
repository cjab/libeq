use std::collections::HashMap;

#[derive(Debug)]
pub struct StringHash(HashMap<usize, String>);

const XOR_KEY: [u8; 8] = [0x95, 0x3a, 0xc5, 0x2a, 0x95, 0x7a, 0x95, 0x6a];

pub fn decode_string(encoded_data: &[u8]) -> String {
    String::from_utf8(
        encoded_data
            .iter()
            .zip(XOR_KEY.iter().cycle())
            .map(|(encoded_char, key_char)| encoded_char ^ key_char)
            .collect(),
    )
    .expect("Invalid data in encoded string")
    .trim_end_matches('\u{0}')
    .to_string()
}

impl StringHash {
    pub fn new(encoded_data: &[u8]) -> StringHash {
        let decoded_string = decode_string(encoded_data);
        let strings = decoded_string.split("\0");
        let indices = decoded_string.match_indices("\0");

        let mut reference_idx = 0;
        let mut hash = HashMap::new();
        for (string, (end_idx, _)) in strings.zip(indices) {
            hash.insert(reference_idx, string.to_string());
            reference_idx = end_idx + 1;
        }
        StringHash(hash)
    }

    pub fn get(&self, name_reference: i32) -> Option<&str> {
        if name_reference.is_negative() {
            let key = name_reference.abs() as usize;
            self.0.get(&key).map(|s| s.as_ref())
        } else {
            None
        }
    }
}
