//! Builds .wld files byte by byte for tests, laid out as the format lays them
//! out, so tests need no game data and don't lean on this crate's writers.

use super::strings::encode_string;

pub(crate) struct WldBuilder {
    version: u32,
    strings: String,
    fragments: Vec<(u32, Vec<u8>)>,
}

impl WldBuilder {
    pub(crate) fn new(version: u32) -> Self {
        Self {
            version,
            strings: "\0".to_owned(),
            fragments: Vec::new(),
        }
    }

    /// Adds a name to the string hash and returns its (negative) reference.
    pub(crate) fn name(&mut self, name: &str) -> i32 {
        let reference = -(self.strings.len() as i32);
        self.strings.push_str(name);
        self.strings.push('\0');
        reference
    }

    /// Adds a fragment, padded to 4 bytes, and returns its 1-based index.
    pub(crate) fn fragment(&mut self, fragment_type: u32, mut body: Vec<u8>) -> i32 {
        body.resize(body.len().div_ceil(4) * 4, 0);
        self.fragments.push((fragment_type, body));
        self.fragments.len() as i32
    }

    /// Adds a fragment exactly as given, with no padding added.
    pub(crate) fn unpadded_fragment(&mut self, fragment_type: u32, body: Vec<u8>) {
        self.fragments.push((fragment_type, body));
    }

    pub(crate) fn build(&self) -> Vec<u8> {
        let mut strings = encode_string(&self.strings);
        strings.resize(strings.len().div_ceil(4) * 4, 0);
        let largest = self.fragments.iter().map(|f| f.1.len()).max().unwrap_or(0);
        let mut out = Vec::new();
        for word in [
            0x5450_3d02,
            self.version,
            self.fragments.len() as u32,
            0,
            largest as u32,
            strings.len() as u32,
            self.strings.matches('\0').count() as u32,
        ] {
            out.extend(word.to_le_bytes());
        }
        out.extend(strings);
        for (fragment_type, body) in &self.fragments {
            out.extend((body.len() as u32).to_le_bytes());
            out.extend(fragment_type.to_le_bytes());
            out.extend(body);
        }
        out.extend([0xff; 4]);
        out
    }
}

/// A 0x30 material with no texture and the given render method.
pub(crate) fn material(name: i32, render_method: u32) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend(name.to_le_bytes());
    body.extend(0u32.to_le_bytes()); // flags: no pair
    body.extend(render_method.to_le_bytes());
    body.extend(0u32.to_le_bytes()); // rgb pen
    body.extend(1.0f32.to_le_bytes()); // brightness
    body.extend(1.0f32.to_le_bytes()); // scaled ambient
    body.extend(0i32.to_le_bytes()); // no simple sprite
    body
}

/// A 0x31 material palette listing the given material fragment indices.
pub(crate) fn palette(name: i32, materials: &[i32]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend(name.to_le_bytes());
    body.extend(0u32.to_le_bytes());
    body.extend((materials.len() as u32).to_le_bytes());
    for material in materials {
        body.extend(material.to_le_bytes());
    }
    body
}

/// Texture coordinates as one of the two formats stores them.
pub(crate) enum Coordinates {
    Old([(i16, i16); 3]),
    New([(f32, f32); 3]),
}

/// A 0x36 mesh of one triangle using material 0 of `palette`.
pub(crate) fn triangle(name: i32, palette: i32, coordinates: Coordinates) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend(name.to_le_bytes());
    body.extend(0x0001_8003u32.to_le_bytes()); // flags
    body.extend(palette.to_le_bytes());
    body.extend([0u8; 12]); // animation, fragment3, fragment4
    body.extend([0u8; 12]); // center
    body.extend([0u8; 12]); // params2
    body.extend(2.0f32.to_le_bytes()); // max distance
    body.extend([0u8; 24]); // min, max
    // Positions, coordinates, normals, colors, faces, skin groups, face groups,
    // vertex groups, meshops, scale.
    for count in [3u16, 3, 3, 0, 1, 0, 1, 1, 0, 0] {
        body.extend(count.to_le_bytes());
    }
    for position in [[0i16, 0, 0], [1, 0, 0], [0, 1, 0]] {
        for axis in position {
            body.extend(axis.to_le_bytes());
        }
    }
    match coordinates {
        Coordinates::Old(pairs) => {
            for (u, v) in pairs {
                body.extend(u.to_le_bytes());
                body.extend(v.to_le_bytes());
            }
        }
        Coordinates::New(pairs) => {
            for (u, v) in pairs {
                body.extend(u.to_le_bytes());
                body.extend(v.to_le_bytes());
            }
        }
    }
    body.extend([0u8, 0, 127].repeat(3)); // normals
    for word in [0u16, 0, 1, 2] {
        body.extend(word.to_le_bytes()); // face flags, then its vertices
    }
    for word in [1u16, 0] {
        body.extend(word.to_le_bytes()); // one face uses material 0
    }
    for word in [3u16, 0] {
        body.extend(word.to_le_bytes()); // three vertices use material 0
    }
    body
}

/// A WLD holding one material, its palette and a triangle using them.
pub(crate) fn one_triangle(version: u32, render_method: u32, coordinates: Coordinates) -> Vec<u8> {
    let mut wld = WldBuilder::new(version);
    let (m, p, t) = (
        wld.name("M_MDF"),
        wld.name("P_MP"),
        wld.name("T_DMSPRITEDEF"),
    );
    let material = wld.fragment(0x30, material(m, render_method));
    let palette = wld.fragment(0x31, palette(p, &[material]));
    wld.fragment(0x36, triangle(t, palette, coordinates));
    wld.build()
}
