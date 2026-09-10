//! Opt-in validation against a supplied client installation; no game data is bundled.
use libeq_eqg::{FormatHeader, identify, zone};
use libeq_pfs::PfsReader;
use std::fs::{self, File};

fn check(name: &str, bytes: &[u8], versions: &mut [usize; 2]) {
    match identify(bytes).expect("read zone header") {
        Some(FormatHeader::TerrainProject) => return,
        Some(FormatHeader::Zone { .. }) => {}
        other => panic!("{name}: expected a zone descriptor, got {other:?}"),
    }
    let parsed = zone::parse(bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert!(
        parsed.trailing_data.is_empty(),
        "{name}: unaccounted trailing data"
    );
    versions[(parsed.version - 1) as usize] += 1;
    for offset in parsed.models.iter().flatten() {
        parsed.string(*offset).unwrap();
    }
    for placement in &parsed.placements {
        parsed.string(placement.name_offset).unwrap();
        assert_eq!(placement.extension_data.len() % 4, 0);
    }
    // Independent fixture counts pin the region/light order and variable records.
    let expected = match name {
        "crescent.zon" => Some((2, 176, 2342, 58, 0, 646120)),
        "anguish.eqg:anguish.zon" => Some((1, 215, 696, 2, 452, 0)),
        "guildhall.zon" => Some((2, 51, 87, 0, 46, 76293)),
        _ => None,
    };
    if let Some(expected) = expected {
        let extensions: usize = parsed
            .placements
            .iter()
            .map(|p| p.extension_words().count())
            .sum();
        assert_eq!(
            (
                parsed.version,
                parsed.models.len(),
                parsed.placements.len(),
                parsed.regions.len(),
                parsed.lights.len(),
                extensions
            ),
            expected,
            "{name}: reference fixture changed"
        );
    }
}

#[test]
#[ignore = "requires LIBEQ_TEST_RAW_DIR containing native binary v1 and v2 zone descriptors"]
fn parses_native_binary_zone_corpus() {
    let root = std::env::var_os("LIBEQ_TEST_RAW_DIR")
        .expect("set LIBEQ_TEST_RAW_DIR to a client installation");
    let root = std::path::Path::new(&root);
    let mut paths: Vec<_> = fs::read_dir(root)
        .expect("read client installation")
        .map(|entry| entry.expect("read directory entry").path())
        .collect();
    paths.sort();
    let mut versions = [0, 0];
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .expect("UTF-8 resource name");
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".zon") {
            check(
                &lower,
                &fs::read(&path).expect("read loose zone descriptor"),
                &mut versions,
            );
        } else if lower.ends_with(".eqg") {
            let mut archive =
                PfsReader::open(File::open(&path).expect("open EQG")).expect("read PFS index");
            let mut members = archive.filenames().expect("read PFS directory");
            members.sort();
            for member in members {
                if member.to_ascii_lowercase().ends_with(".zon") {
                    let bytes = archive
                        .get(&member)
                        .expect("read descriptor")
                        .expect("descriptor exists");
                    check(
                        &format!("{lower}:{}", member.to_ascii_lowercase()),
                        &bytes,
                        &mut versions,
                    );
                }
            }
        }
    }
    assert!(
        versions[0] > 0 && versions[1] > 0,
        "corpus must exercise both binary versions: {versions:?}"
    );
    eprintln!(
        "parsed {} binary zone descriptors: {} v1, {} v2",
        versions.iter().sum::<usize>(),
        versions[0],
        versions[1]
    );
}
