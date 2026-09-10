//! Opt-in mesh validation; game assets are supplied locally and never bundled.
use libeq_eqg::mesh::{self, MeshKind};
use libeq_pfs::PfsReader;
use std::fs::{self, File};

#[test]
#[ignore = "requires LIBEQ_TEST_RAW_DIR with terrain v1/v2/v3 and selected model archives"]
fn parses_native_mesh_corpus() {
    let root = std::env::var_os("LIBEQ_TEST_RAW_DIR")
        .expect("set LIBEQ_TEST_RAW_DIR to a client installation");
    let mut paths: Vec<_> = fs::read_dir(root)
        .expect("read client installation")
        .map(|entry| entry.expect("read directory entry").path())
        .collect();
    paths.sort();
    let mut counts = [[0usize; 3]; 2];
    let mut secondary_uv_files = 0;
    let mut skeletal_files = 0;
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_ascii_lowercase();
        if !name.ends_with(".eqg") {
            continue;
        }
        let models = matches!(
            name.as_str(),
            "crescent.eqg"
                | "guildhall.eqg"
                | "anguish.eqg"
                | "row.eqg"
                | "shi.eqg"
                | "arcstone.eqg"
        );
        let mut archive =
            PfsReader::open(File::open(&path).expect("open EQG")).expect("read PFS index");
        let mut members = archive.filenames().expect("read PFS directory");
        members.sort();
        for member in members {
            let lower = member.to_ascii_lowercase();
            if !lower.ends_with(".ter") && !(models && lower.ends_with(".mod")) {
                continue;
            }
            let bytes = archive
                .get(&member)
                .expect("read mesh")
                .expect("mesh exists");
            let label = format!("{name}:{lower}");
            let mesh = mesh::parse(&bytes).unwrap_or_else(|e| panic!("{label}: {e}"));
            let kind = match mesh.kind {
                MeshKind::Terrain => 0,
                MeshKind::Model => 1,
            };
            counts[kind][(mesh.version - 1) as usize] += 1;
            if mesh.bone_count.unwrap_or(0) == 0 {
                assert!(
                    mesh.trailing_data.is_empty(),
                    "{label}: unexpected static suffix"
                );
            } else {
                skeletal_files += 1;
                assert!(
                    !mesh.trailing_data.is_empty(),
                    "{label}: missing skeletal suffix"
                );
            }
            if mesh.version == 2 && mesh.uv_marker == Some(1) {
                secondary_uv_files += 1;
                assert!(mesh.vertices.iter().all(|v| v.uv1.is_some()), "{label}");
            }
            for material in &mesh.materials {
                mesh.string(material.name_offset).unwrap();
                mesh.string(material.shader_offset).unwrap();
            }
            // Independently measured fixture counts pin geometry boundaries and sentinels.
            let expected = match label.as_str() {
                "crescent.eqg:ter_crescent.ter" => Some((3, 49, 83698, 67484, 1854)),
                "guildhall.eqg:ter_guildhall.ter" => Some((3, 29, 28584, 13803, 160)),
                "fhalls.eqg:ter_temple01.ter" => Some((1, 41, 69459, 41138, 1014)),
                "anguish.eqg:ter_island.ter" => Some((2, 35, 149400, 96395, 480)),
                "row.eqg:row.mod" => Some((1, 3, 393, 256, 0)),
                "anguish.eqg:obj_arch01.mod" => Some((2, 1, 234, 96, 0)),
                "arcstone.eqg:obj_arcportal.mod" => Some((3, 5, 96, 166, 0)),
                _ => None,
            };
            let first_vertex = match label.as_str() {
                "crescent.eqg:ter_crescent.ter" => Some((
                    [3302307367, 3306407300, 3276396736],
                    [1063567394, 1032699765, 3202555220],
                    Some(4252991359),
                    [1107558399, 3212836864],
                    Some([1089538037, 3189213624]),
                )),
                "anguish.eqg:ter_island.ter" => Some((
                    [1119447288, 1140247283, 3281090687],
                    [3205821111, 1048856445, 3208997990],
                    None,
                    [1057956895, 3222027149],
                    None,
                )),
                "fhalls.eqg:ter_temple01.ter" => Some((
                    [0, 2861424279, 3241148416],
                    [2985713409, 2978374239, 1065353216],
                    None,
                    [1056964606, 3204448264],
                    None,
                )),
                _ => None,
            };
            if let Some(expected) = first_vertex {
                let vertex = &mesh.vertices[0];
                assert_eq!(
                    (
                        vertex.position.map(f32::to_bits),
                        vertex.normal.map(f32::to_bits),
                        vertex.color,
                        vertex.uv0.map(f32::to_bits),
                        vertex.uv1.map(|uv| uv.map(f32::to_bits))
                    ),
                    expected,
                    "{label}: reference vertex changed"
                );
            }
            if let Some(expected) = expected {
                assert_eq!(
                    (
                        mesh.version,
                        mesh.materials.len(),
                        mesh.vertices.len(),
                        mesh.triangles.len(),
                        mesh.triangles
                            .iter()
                            .filter(|triangle| triangle.material_index == u32::MAX)
                            .count()
                    ),
                    expected,
                    "{label}: reference fixture changed"
                );
            }
        }
    }
    assert!(
        counts.iter().flatten().all(|&n| n > 0),
        "corpus must exercise both mesh kinds at all three versions: {counts:?}"
    );
    assert!(secondary_uv_files > 0, "corpus needs v2 secondary UV data");
    assert!(
        skeletal_files > 0,
        "corpus needs a model with opaque skeletal data"
    );
    eprintln!(
        "parsed mesh corpus: terrain v1/v2/v3 {:?}, model v1/v2/v3 {:?}; {secondary_uv_files} v2 secondary-UV files, {skeletal_files} skeletal suffixes",
        counts[0], counts[1]
    );
}
