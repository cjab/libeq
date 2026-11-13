use libeq_wld::parser::{Actor, ActorDef, FragmentType, StringReference, WldDoc};

// Offsets: empty = 0, ALIAS = 1, MODEL = 7, NAMED = 13.
const STRINGS: &[u8] = b"\0ALIAS\0MODEL\0NAMED\0\0";

fn fragment(kind: u32, words: &[i32]) -> Vec<u8> {
    let body: Vec<_> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    [
        (body.len() as u32).to_le_bytes().as_slice(),
        kind.to_le_bytes().as_slice(),
        &body,
    ]
    .concat()
}

fn fixture(reference: i32, actor_def: bool) -> Vec<u8> {
    // Slot 1 is either a valid ActorDef named MODEL or an unrelated Sphere.
    let target = if actor_def {
        fragment(0x14, &[-7, 0, 0, 0, 0, 0, 0])
    } else {
        fragment(0x16, &[0, 0])
    };
    // Actor: no optional fields, null sphere reference and empty user data.
    let actor = fragment(0x15, &[0, reference, 0, 0, 0]);
    let key = [0x95, 0x3a, 0xc5, 0x2a, 0x95, 0x7a, 0x95, 0x6a];
    let strings: Vec<_> = STRINGS
        .iter()
        .zip(key.iter().cycle())
        .map(|(byte, key)| byte ^ key)
        .collect();
    let header: Vec<_> = [0x5450_3d02_u32, 0x0001_5500, 2, 0, 0, 20, 5]
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    [header, strings, target, actor].concat()
}

#[test]
fn positive_actor_def_index_does_not_alias_a_string_offset() {
    let bytes = fixture(1, true);
    let doc = WldDoc::parse(&bytes).unwrap();
    let Some(FragmentType::ActorDef(target)) = doc.at(0) else {
        panic!("fixture must contain a typed ActorDef at reference 1");
    };
    assert_eq!(doc.get_string(target.name_reference), Some("MODEL"));
    assert_eq!(doc.get_string(StringReference(1)), Some("ALIAS"));
    assert_eq!(doc.fragment_iter::<ActorDef>().count(), 1);

    let wld = libeq_wld::load(&bytes).unwrap();
    let mut objects = wld.objects();
    assert_eq!(objects.next().unwrap().model_name(), None);
    assert!(objects.next().is_none());
}

#[test]
fn positive_missing_and_wrong_kind_indexes_are_unsupported() {
    // Reference 1 points to a Sphere; reference 7 is missing but aliases MODEL.
    for reference in [1, 7, i32::MAX] {
        let bytes = fixture(reference, false);
        let wld = libeq_wld::load(&bytes).unwrap();
        assert_eq!(
            wld.objects().next().unwrap().model_name(),
            None,
            "positive reference {reference}"
        );
    }
}

#[test]
fn nonpositive_references_keep_existing_string_lookup() {
    for (reference, expected) in [
        (-13, Some("NAMED")),
        (-1, Some("ALIAS")),
        (0, Some("")),
        (-2, None),
        (-1_000_000, None),
        (i32::MIN, None),
    ] {
        let bytes = fixture(reference, true);
        let wld = libeq_wld::load(&bytes).unwrap();
        assert_eq!(
            wld.objects().next().unwrap().model_name(),
            expected,
            "nonpositive reference {reference}"
        );
    }
}

#[test]
fn parsing_and_serialization_preserve_the_raw_reference_value() {
    for reference in [1, 7, i32::MAX, -13, 0, i32::MIN] {
        let bytes = fixture(reference, true);
        let doc = WldDoc::parse(&bytes).unwrap();
        assert_eq!(
            doc.fragment_iter::<Actor>()
                .next()
                .unwrap()
                .actor_def_reference,
            StringReference(reference)
        );
        let serialized = doc.to_bytes();
        let reparsed = WldDoc::parse(&serialized).unwrap();
        assert_eq!(
            reparsed
                .fragment_iter::<Actor>()
                .next()
                .unwrap()
                .actor_def_reference,
            StringReference(reference)
        );
    }
}
