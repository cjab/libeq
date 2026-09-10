//! Raw EQGT terrain and EQGM model geometry, versions 1 through 3.
//! Bone records and other suffix data remain opaque. No coordinate conversion,
//! material interpretation, or skeletal animation is performed.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("invalid EQGT/EQGM magic")]
    InvalidMagic,
    #[error("unsupported mesh version {version}")]
    UnsupportedVersion { version: u32 },
    #[error("truncated {context} at byte {offset}: need {needed} bytes, have {remaining}")]
    Truncated {
        context: &'static str,
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    #[error("byte count overflow in {context}")]
    CountOverflow { context: &'static str },
    #[error("invalid or unterminated string reference {offset}")]
    InvalidStringReference { offset: u32 },
    #[error("vertex index {index} is outside vertex count {vertex_count}")]
    InvalidVertexReference { index: u32, vertex_count: u32 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshKind {
    Terrain,
    Model,
}
#[derive(Debug)]
pub struct Mesh<'a> {
    pub kind: MeshKind,
    pub version: u32,
    pub string_table: &'a [u8],
    pub materials: Vec<Material>,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
    /// Model header count only; bone records are not decoded or validated.
    pub bone_count: Option<u32>,
    /// Present only in version 2, including unrecognized marker values.
    pub uv_marker: Option<u32>,
    /// Bytes following geometry, including any model bone records.
    pub trailing_data: &'a [u8],
}
impl Mesh<'_> {
    /// Resolve a NUL-terminated byte string, excluding the terminator.
    /// Non-UTF-8 bytes and suffix references are permitted.
    pub fn string(&self, offset: u32) -> Result<&[u8], ParseError> {
        let bytes = usize::try_from(offset)
            .ok()
            .and_then(|n| self.string_table.get(n..))
            .ok_or(ParseError::InvalidStringReference { offset })?;
        let end = bytes
            .iter()
            .position(|&b| b == 0)
            .ok_or(ParseError::InvalidStringReference { offset })?;
        Ok(&bytes[..end])
    }
}
#[derive(Debug)]
pub struct Material {
    pub index: u32,
    pub name_offset: u32,
    pub shader_offset: u32,
    pub properties: Vec<Property>,
}
#[derive(Debug)]
pub struct Property {
    pub name_offset: u32,
    /// Raw type tag. Type 2 values are validated as string offsets.
    pub kind: u32,
    pub value: u32,
}
/// Floating-point bit patterns are retained, including non-finite values.
#[derive(Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    /// Packed version 3 color; absent in versions 1 and 2.
    pub color: Option<u32>,
    pub uv0: [f32; 2],
    pub uv1: Option<[f32; 2]>,
}
#[derive(Debug)]
pub struct Triangle {
    pub vertex_indices: [u32; 3],
    /// Raw material reference, including sentinels and out-of-range values.
    pub material_index: u32,
    pub flags: u32,
}
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn require(&self, needed: usize, context: &'static str) -> Result<(), ParseError> {
        let remaining = self.bytes.len() - self.offset;
        if needed > remaining {
            return Err(ParseError::Truncated {
                context,
                offset: self.offset,
                needed,
                remaining,
            });
        }
        Ok(())
    }
    fn take(&mut self, n: usize, context: &'static str) -> Result<&'a [u8], ParseError> {
        self.require(n, context)?;
        let start = self.offset;
        self.offset += n;
        Ok(&self.bytes[start..self.offset])
    }
    fn word(&mut self, context: &'static str) -> Result<u32, ParseError> {
        let b = self.take(4, context)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn words<const N: usize>(&mut self, context: &'static str) -> Result<[u32; N], ParseError> {
        let mut result = [0; N];
        for v in &mut result {
            *v = self.word(context)?;
        }
        Ok(result)
    }
}
fn size(count: u32, stride: usize) -> Result<usize, ParseError> {
    usize::try_from(count)
        .ok()
        .and_then(|n| n.checked_mul(stride))
        .ok_or(ParseError::CountOverflow {
            context: "record counts",
        })
}
/// Decode geometry, retaining any suffix without interpreting skeletal records.
/// Counted byte requirements are checked before allocating record vectors.
/// Every string reference must terminate within the table and every triangle
/// vertex reference must be in range. Material references remain uninterpreted.
pub fn parse(input: &[u8]) -> Result<Mesh<'_>, ParseError> {
    let mut r = Reader {
        bytes: input,
        offset: 0,
    };
    let kind = match r.take(4, "magic")? {
        b"EQGT" => MeshKind::Terrain,
        b"EQGM" => MeshKind::Model,
        _ => return Err(ParseError::InvalidMagic),
    };
    let version = r.word("version")?;
    if !(1..=3).contains(&version) {
        return Err(ParseError::UnsupportedVersion { version });
    }
    let [strings, materials, vertices, triangles] = r.words("header")?;
    let bone_count = if kind == MeshKind::Model {
        Some(r.word("bone count")?)
    } else {
        None
    };
    let mut minimum = if version == 2 { 4usize } else { 0 };
    for (count, stride) in [
        (strings, 1),
        (materials, 16),
        (vertices, if version == 3 { 44 } else { 32 }),
        (triangles, 20),
    ] {
        minimum = minimum
            .checked_add(size(count, stride)?)
            .ok_or(ParseError::CountOverflow {
                context: "record counts",
            })?;
    }
    r.require(minimum, "counted records")?;
    let string_table = r.take(size(strings, 1)?, "string table")?;
    let last_nul = string_table.iter().rposition(|&b| b == 0);
    let check = |offset: u32| {
        if usize::try_from(offset)
            .ok()
            .zip(last_nul)
            .is_some_and(|(n, last)| n <= last)
        {
            Ok(())
        } else {
            Err(ParseError::InvalidStringReference { offset })
        }
    };
    let mut mesh = Mesh {
        kind,
        version,
        string_table,
        materials: Vec::new(),
        vertices: Vec::new(),
        triangles: Vec::new(),
        bone_count,
        uv_marker: None,
        trailing_data: &[],
    };
    for _ in 0..materials {
        let [index, name_offset, shader_offset, count] = r.words("material")?;
        check(name_offset)?;
        check(shader_offset)?;
        r.require(size(count, 12)?, "material properties")?;
        let mut properties = Vec::new();
        for _ in 0..count {
            let [name_offset, kind, value] = r.words("property")?;
            check(name_offset)?;
            if kind == 2 {
                check(value)?;
            }
            properties.push(Property {
                name_offset,
                kind,
                value,
            });
        }
        mesh.materials.push(Material {
            index,
            name_offset,
            shader_offset,
            properties,
        });
    }
    r.require(
        size(vertices, if version == 3 { 44 } else { 32 })?,
        "vertices",
    )?;
    for _ in 0..vertices {
        let position = r.words("position")?.map(f32::from_bits);
        let normal = r.words("normal")?.map(f32::from_bits);
        let color = if version == 3 {
            Some(r.word("color")?)
        } else {
            None
        };
        let uv0 = r.words("primary UV")?.map(f32::from_bits);
        let uv1 = if version == 3 {
            Some(r.words("secondary UV")?.map(f32::from_bits))
        } else {
            None
        };
        mesh.vertices.push(Vertex {
            position,
            normal,
            color,
            uv0,
            uv1,
        });
    }
    r.require(size(triangles, 20)?, "triangles")?;
    for _ in 0..triangles {
        let vertex_indices = r.words("triangle vertices")?;
        for index in vertex_indices {
            if index >= vertices {
                return Err(ParseError::InvalidVertexReference {
                    index,
                    vertex_count: vertices,
                });
            }
        }
        mesh.triangles.push(Triangle {
            vertex_indices,
            material_index: r.word("triangle material")?,
            flags: r.word("triangle flags")?,
        });
    }
    if version == 2 {
        let marker = r.word("UV marker")?;
        mesh.uv_marker = Some(marker);
        if marker == 1 || (kind == MeshKind::Terrain && marker == 2) {
            r.require(size(vertices, 8)?, "secondary UVs")?;
            for vertex in &mut mesh.vertices {
                vertex.uv1 = Some(r.words("secondary UV")?.map(f32::from_bits));
            }
        }
    }
    mesh.trailing_data = &input[r.offset..];
    Ok(mesh)
}
