use libeq_eqg::{Error, FormatHeader, identify};

#[test]
fn identifies_binary_versions_without_assuming_support() {
    for version in [0, 1, 2, 3, u32::MAX] {
        for (magic, expected) in [
            (b"EQGZ", FormatHeader::Zone { version }),
            (b"EQGT", FormatHeader::Terrain { version }),
            (b"EQGM", FormatHeader::Model { version }),
        ] {
            let mut input = magic.to_vec();
            input.extend(version.to_le_bytes());
            assert_eq!(identify(&input), Ok(Some(expected)));
            input.extend(b"arbitrary body");
            assert_eq!(identify(&input), Ok(Some(expected)));
        }
    }
}

#[test]
fn decodes_little_endian_version() {
    assert_eq!(
        identify(b"EQGZ\x01\x02\x03\x04"),
        Ok(Some(FormatHeader::Zone {
            version: 0x04030201
        }))
    );
}

#[test]
fn identifies_text_signatures_without_a_binary_version() {
    for (input, expected) in [
        (b"EQTZP".as_slice(), FormatHeader::TerrainProject),
        (b"EQOBG".as_slice(), FormatHeader::ObjectGroup),
    ] {
        assert_eq!(identify(input), Ok(Some(expected)));
        let mut body = input.to_vec();
        body.extend(b"\r\n*NAME example\r\n");
        assert_eq!(identify(&body), Ok(Some(expected)));
    }
}

#[test]
fn every_incomplete_nonempty_header_prefix_is_truncated() {
    for complete in [
        b"EQGZ\x01\0\0\0".as_slice(),
        b"EQGT\x02\0\0\0".as_slice(),
        b"EQGM\x03\0\0\0".as_slice(),
        b"EQTZP".as_slice(),
        b"EQOBG".as_slice(),
    ] {
        for actual in 1..complete.len() {
            assert!(
                matches!(identify(&complete[..actual]),
                    Err(Error::TruncatedHeader { expected, actual: found })
                        if expected > actual && found == actual),
                "{complete:?} at {actual}"
            );
        }
    }
    assert_eq!(
        identify(b"EQGZ\x01"),
        Err(Error::TruncatedHeader {
            expected: 8,
            actual: 5
        })
    );
}

#[test]
fn unrecognized_data_is_not_a_truncation_error() {
    for input in [
        b"".as_slice(),
        b"X",
        b"EX",
        b"EQX",
        b"EQGX",
        b"EQTX",
        b"EQOBX",
        b"eqgz\0\0\0\0",
        b"\xef\xbb\xbfEQTZP",
    ] {
        assert_eq!(identify(input), Ok(None), "{input:?}");
    }
}
