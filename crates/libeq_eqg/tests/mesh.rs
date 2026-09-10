use libeq_eqg::mesh::{self, MeshKind, ParseError};
fn word(b: &mut Vec<u8>, v: u32) {
    b.extend(v.to_le_bytes());
}
fn fixture(model: bool, version: u32, marker: u32) -> Vec<u8> {
    let mut b = if model {
        b"EQGM".to_vec()
    } else {
        b"EQGT".to_vec()
    };
    for v in [version, 4, 1, 1, 1] {
        word(&mut b, v);
    }
    if model {
        word(&mut b, 7);
    }
    b.extend(b"a\0\xff\0");
    for v in [99, 0, 2, 2, 0, 2, 2, 2, 77, 0xdeadbeef] {
        word(&mut b, v);
    }
    for v in [0x7fc01234, 2, 3, 4, 5, 6] {
        word(&mut b, v);
    }
    if version == 3 {
        word(&mut b, 0x12345678);
    }
    for v in [7, 8] {
        word(&mut b, v);
    }
    if version == 3 {
        for v in [9, 10] {
            word(&mut b, v);
        }
    }
    for v in [0, 0, 0, u32::MAX, 0xfedcba98] {
        word(&mut b, v);
    }
    if version == 2 {
        word(&mut b, marker);
        if marker == 1 || (!model && marker == 2) {
            for v in [9, 10] {
                word(&mut b, v);
            }
        }
    }
    b
}
#[test]
fn versions_and_kinds_preserve_raw_fields() {
    for model in [false, true] {
        for version in 1..=3 {
            let mut b = fixture(model, version, 1);
            b.extend([123, 45]);
            let m = mesh::parse(&b).unwrap();
            assert_eq!(
                m.kind,
                if model {
                    MeshKind::Model
                } else {
                    MeshKind::Terrain
                }
            );
            assert_eq!(m.bone_count, model.then_some(7));
            assert_eq!(m.version, version);
            assert_eq!(m.string(2).unwrap(), &[255]);
            assert_eq!(m.materials[0].index, 99);
            assert_eq!(m.materials[0].properties[1].value, 0xdeadbeef);
            assert_eq!(m.materials[0].properties[1].kind, 77);
            assert_eq!(m.vertices[0].position[0].to_bits(), 0x7fc01234);
            assert_eq!(m.vertices[0].color, (version == 3).then_some(0x12345678));
            assert_eq!(m.vertices[0].uv0[0].to_bits(), 7);
            assert_eq!(
                m.vertices[0].uv1.map(|v| v.map(f32::to_bits)),
                (version >= 2).then_some([9, 10])
            );
            assert_eq!(m.triangles[0].material_index, u32::MAX);
            assert_eq!(m.triangles[0].flags, 0xfedcba98);
            assert_eq!(m.trailing_data, [123, 45]);
        }
    }
}
#[test]
fn version_two_marker_distinguishes_terrain_and_model() {
    for model in [false, true] {
        for marker in [0, 1, 2, 99] {
            let mut b = fixture(model, 2, marker);
            b.extend([1, 2, 3]);
            let m = mesh::parse(&b).unwrap();
            assert_eq!(m.uv_marker, Some(marker));
            assert_eq!(
                m.vertices[0].uv1.is_some(),
                marker == 1 || (!model && marker == 2)
            );
            assert_eq!(m.trailing_data, [1, 2, 3]);
        }
    }
}
#[test]
fn truncations_and_invalid_references_are_rejected() {
    for model in [false, true] {
        for version in 1..=3 {
            let b = fixture(model, version, 1);
            for n in 0..b.len() {
                assert!(mesh::parse(&b[..n]).is_err(), "{model}/{version}/{n}");
            }
        }
    }
    let b = fixture(false, 1, 0);
    for offset in [8, 12, 16, 20, 40] {
        let mut bad = b.clone();
        bad[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(mesh::parse(&bad).is_err(), "count {offset}");
    }
    for offset in [32, 36, 44, 52] {
        let mut bad = b.clone();
        bad[offset..offset + 4].copy_from_slice(&4u32.to_le_bytes());
        assert!(
            matches!(
                mesh::parse(&bad),
                Err(ParseError::InvalidStringReference { .. })
            ),
            "string {offset}"
        );
    }
    let mut bad = b.clone();
    bad[100..104].copy_from_slice(&1u32.to_le_bytes());
    assert!(matches!(
        mesh::parse(&bad),
        Err(ParseError::InvalidVertexReference { .. })
    ));
    let mut bad = b.clone();
    bad[4..8].copy_from_slice(&4u32.to_le_bytes());
    assert!(matches!(
        mesh::parse(&bad),
        Err(ParseError::UnsupportedVersion { version: 4 })
    ));
    let mut bad = b;
    bad[0] = 0;
    assert!(matches!(mesh::parse(&bad), Err(ParseError::InvalidMagic)));
}

#[test]
fn empty_meshes_and_string_suffixes() {
    for model in [false, true] {
        for version in 1..=3 {
            let mut b = if model {
                b"EQGM".to_vec()
            } else {
                b"EQGT".to_vec()
            };
            for v in [version, 0, 0, 0, 0] {
                word(&mut b, v);
            }
            if model {
                word(&mut b, 0);
            }
            if version == 2 {
                word(&mut b, 1);
            }
            let m = mesh::parse(&b).unwrap();
            assert!(m.vertices.is_empty());
            assert!(m.trailing_data.is_empty());
            assert!(m.string(0).is_err());
        }
    }
    let mut b = fixture(false, 1, 0);
    b[32..36].copy_from_slice(&1u32.to_le_bytes());
    b[112..116].copy_from_slice(&123u32.to_le_bytes());
    let m = mesh::parse(&b).unwrap();
    assert_eq!(m.string(m.materials[0].name_offset).unwrap(), b"");
    assert_eq!(m.triangles[0].material_index, 123);
    b[27] = 1;
    assert!(matches!(
        mesh::parse(&b),
        Err(ParseError::InvalidStringReference { .. })
    ));
}
