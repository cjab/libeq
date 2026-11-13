use libeq_wld::parser::{Fragment, FragmentParser, SimpleSpriteDef};

fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

#[test]
fn current_frame_survives_parse_and_serialize() {
    let bytes = words(&[0, 0x38, 2, 1, 137, 1, 2]);
    let (remaining, sprite) = SimpleSpriteDef::parse(&bytes).unwrap();
    assert!(remaining.is_empty());
    assert_eq!(sprite.current_frame, Some(1));
    assert_eq!(sprite.sleep, Some(137));
    assert_eq!(sprite.to_bytes(), bytes);
    let round_trip = SimpleSpriteDef::parse(&sprite.to_bytes()).unwrap().1;
    assert_eq!(round_trip, sprite);
}

fn animation_document(flags: u32, current: Option<u32>, sleep: Option<u32>) -> Vec<u8> {
    use libeq_wld::parser::{BmInfo, EncodedFilename, StringReference};
    let animated = flags & 0x08 != 0;
    let frames = if animated { 2 } else { 1 };
    let mut definition = words(&[0, flags, frames]);
    if let Some(current) = current { definition.extend_from_slice(&current.to_le_bytes()); }
    if let Some(sleep) = sleep { definition.extend_from_slice(&sleep.to_le_bytes()); }
    definition.extend_from_slice(&1u32.to_le_bytes());
    if animated { definition.extend_from_slice(&2u32.to_le_bytes()); }
    let bitmap = |name: &str| BmInfo {
        name_reference: StringReference::new(0), entry_count: 0,
        entries: vec![EncodedFilename { name_length: (name.len() + 1) as u16, file_name: name.into() }],
    }.to_bytes();
    let fragments = [
        (0x03, bitmap("FRAME_A.BMP")),
        (0x03, bitmap("FRAME_B.BMP")),
        (0x04, definition),
        (0x05, words(&[0, 3, 0x50])),
        (0x30, words(&[0, 0, 0x8000_0001, 0, 1.0f32.to_bits(), 1.0f32.to_bits(), 4])),
    ];
    let mut document = words(&[0x5450_3d02, 0x0001_5500, 5, 0, 0, 1, 0]);
    document.push(0x95); // Encoded null string; all fragment names are absent.
    for (kind, body) in fragments {
        document.extend_from_slice(&(body.len() as u32).to_le_bytes());
        document.extend_from_slice(&(kind as u32).to_le_bytes());
        document.extend_from_slice(&body);
    }
    document
}

#[test]
fn public_texture_metadata_preserves_presence_and_zero_without_defaults() {
    for (flags, current, sleep) in [
        (0x38, Some(1), Some(137)),
        (0x08, None, None),
        (0x00, None, None),
        (0x20, Some(0), None),
        (0x18, None, Some(0)),
        (0x30, Some(0), None),
        (0x10, None, None),
        (0x28, Some(1), None),
        (0x38, Some(u32::MAX), Some(u32::MAX)),
    ] {
        let bytes = animation_document(flags, current, sleep);
        let wld = libeq_wld::load(&bytes).unwrap();
        let material = wld.materials().next().unwrap();
        let texture = material.base_color_texture().unwrap();
        assert_eq!(texture.current_frame(), current, "flags {flags:#x}");
        assert_eq!(texture.sleep(), sleep, "flags {flags:#x}");
        let expected = if flags & 0x08 != 0 {
            vec!["frame_a.bmp", "frame_b.bmp"]
        } else { vec!["frame_a.bmp"] };
        assert_eq!(texture.iter_sources().collect::<Vec<_>>(), expected, "flags {flags:#x}");
    }
}
