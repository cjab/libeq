use libeq_eqg::zone::{ParseError, parse};

fn word(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_le_bytes());
}
fn fixture(version: u32) -> Vec<u8> {
    let mut out = b"EQGZ".to_vec();
    for value in [version, 9, 2, 2, 1, 1] {
        word(&mut out, value);
    }
    out.extend(b"mesh\0\xffnm\0");
    word(&mut out, 0);
    word(&mut out, u32::MAX);
    for index in [1, u32::MAX] {
        word(&mut out, index);
        word(&mut out, 5);
        for value in [0x7fc01234, 2, 3, 4, 5, 6, 7] {
            word(&mut out, value);
        }
        if version == 2 {
            word(&mut out, if index == 1 { 2 } else { 0 });
            if index == 1 {
                word(&mut out, 0x12345678);
                word(&mut out, 0xffffffff);
            }
        }
    }
    word(&mut out, 1);
    for value in 10..19 {
        word(&mut out, value);
    }
    word(&mut out, 8);
    for value in 20..27 {
        word(&mut out, value);
    }
    out
}

#[test]
fn parses_versions_and_preserves_raw_records() {
    for version in [1, 2] {
        let mut bytes = fixture(version);
        bytes.extend(b"tail");
        let zone = parse(&bytes).unwrap();
        assert_eq!(zone.version, version);
        assert_eq!(zone.models, [Some(0), None]);
        assert_eq!(zone.placements.len(), 2);
        let first = &zone.placements[0];
        assert_eq!(first.model_index, Some(1));
        assert_eq!(zone.placements[1].model_index, None);
        assert_eq!(first.name_offset, 5);
        assert_eq!(first.position.map(f32::to_bits), [0x7fc01234, 2, 3]);
        assert_eq!(first.rotation.map(f32::to_bits), [4, 5, 6]);
        assert_eq!(first.scale.to_bits(), 7);
        assert_eq!(
            first.extension_words().collect::<Vec<_>>(),
            if version == 2 {
                vec![0x12345678, u32::MAX]
            } else {
                vec![]
            }
        );
        assert!(zone.placements[1].extension_data.is_empty());
        assert_eq!(zone.regions[0].name_offset, 1);
        assert_eq!(zone.regions[0].data, [10, 11, 12, 13, 14, 15, 16, 17, 18]);
        assert_eq!(zone.lights[0].data, [20, 21, 22, 23, 24, 25, 26]);
        assert_eq!(zone.string(1).unwrap(), b"esh");
        assert_eq!(zone.string(5).unwrap(), b"\xffnm");
        assert_eq!(zone.string(8).unwrap(), b"");
        assert!(matches!(
            zone.string(9),
            Err(ParseError::InvalidStringReference { .. })
        ));
        assert_eq!(zone.trailing_data, b"tail");
    }
}

#[test]
fn every_truncated_prefix_is_rejected() {
    for version in [1, 2] {
        let bytes = fixture(version);
        for end in 0..bytes.len() {
            assert!(
                parse(&bytes[..end]).is_err(),
                "version {version}, prefix {end}"
            );
        }
    }
}

#[test]
fn rejects_invalid_references() {
    let valid = fixture(1);
    // Model offset, placement name, region name, light name.
    for offset in [37, 49, 117, 157] {
        let mut bytes = valid.clone();
        bytes[offset..offset + 4].copy_from_slice(&9u32.to_le_bytes());
        assert!(
            matches!(
                parse(&bytes),
                Err(ParseError::InvalidStringReference { .. })
            ),
            "offset {offset}"
        );
    }
    let mut bytes = valid.clone();
    bytes[45..49].copy_from_slice(&2u32.to_le_bytes());
    assert!(matches!(
        parse(&bytes),
        Err(ParseError::InvalidModelReference { .. })
    ));
    let mut bytes = valid;
    bytes[36] = b'x';
    assert!(matches!(
        parse(&bytes),
        Err(ParseError::InvalidStringReference { .. })
    ));
}

#[test]
fn validates_magic_versions_and_counts() {
    let mut bytes = fixture(1);
    bytes[0] = b'x';
    assert!(matches!(parse(&bytes), Err(ParseError::InvalidMagic)));
    bytes[0] = b'E';
    bytes[4..8].copy_from_slice(&3u32.to_le_bytes());
    assert!(matches!(
        parse(&bytes),
        Err(ParseError::UnsupportedVersion { version: 3 })
    ));
    for offset in [8, 12, 16, 20, 24] {
        let mut bytes = fixture(1);
        bytes[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse(&bytes).is_err());
    }
    let mut bytes = fixture(2);
    bytes[81..85].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(parse(&bytes).is_err());
}

#[test]
fn empty_zone_and_unreferenced_unterminated_table_are_allowed() {
    let mut bytes = b"EQGZ".to_vec();
    for value in [1, 3, 0, 0, 0, 0] {
        word(&mut bytes, value);
    }
    bytes.extend(b"abc");
    let zone = parse(&bytes).unwrap();
    assert!(zone.models.is_empty());
    assert!(zone.string(0).is_err());
}

#[test]
fn region_and_light_counts_select_distinct_record_layouts() {
    let mut bytes = b"EQGZ".to_vec();
    for value in [1, 1, 0, 0, 2, 1] {
        word(&mut bytes, value);
    }
    bytes.push(0);
    for marker in [100, 200] {
        word(&mut bytes, 0);
        for value in marker..marker + 9 {
            word(&mut bytes, value);
        }
    }
    word(&mut bytes, 0);
    for value in 300..307 {
        word(&mut bytes, value);
    }
    let zone = parse(&bytes).unwrap();
    assert_eq!(zone.regions.len(), 2);
    assert_eq!(zone.lights.len(), 1);
    assert_eq!(
        zone.regions[0].data,
        [100, 101, 102, 103, 104, 105, 106, 107, 108]
    );
    assert_eq!(
        zone.regions[1].data,
        [200, 201, 202, 203, 204, 205, 206, 207, 208]
    );
    assert_eq!(zone.lights[0].data, [300, 301, 302, 303, 304, 305, 306]);
    assert!(zone.trailing_data.is_empty());
}
