mod parser;

pub use parser::Error;
use parser::{MaterialFragment, MeshFragment, TextureFragment, WldDoc};

pub struct Wld<'a>(WldDoc<'a>);

/// Load and parse a wld file from a slice.
pub fn load(data: &[u8]) -> Result<Wld, Error> {
    Wld::load(data)
}

impl<'a> Wld<'a> {
    fn load(data: &[u8]) -> Result<Wld, Error> {
        WldDoc::parse(&data[..]).map(|(_, wld_doc)| Ok(Wld(wld_doc)))?
    }

    /// Iterate over all meshes in the wld file.
    pub fn meshes(&self) -> impl Iterator<Item = Mesh> + '_ {
        self.0.meshes().map(move |(name, fragment)| Mesh {
            doc: &self.0,
            name,
            fragment,
        })
    }

    /// Iterate over all materials in the wld file.
    pub fn materials(&self) -> impl Iterator<Item = Material> + '_ {
        self.0.materials().map(move |(name, fragment)| Material {
            doc: &self.0,
            name,
            fragment,
        })
    }
}

#[derive(Debug)]
pub struct Mesh<'a> {
    doc: &'a WldDoc<'a>,
    fragment: MeshFragment,
    name: Option<&'a str>,
}

impl<'a> Mesh<'a> {
    /// The name of the mesh
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// The positions of the vertices that make up this mesh.
    pub fn positions(&self) -> Vec<[f32; 3]> {
        let center = self.fragment.center;
        let scale = self.fragment.scale;
        self.fragment
            .positions
            .iter()
            .map(|v| {
                [
                    (v.0 as f32 - center.0) as f32 / ((1 << scale) as f32),
                    (v.1 as f32 - center.1) as f32 / ((1 << scale) as f32),
                    (v.2 as f32 - center.2) as f32 / ((1 << scale) as f32),
                ]
            })
            .collect()
    }

    /// The vertex normals of the mesh.
    pub fn normals(&self) -> Vec<[f32; 3]> {
        self.fragment
            .vertex_normals
            .iter()
            .map(|v| {
                [
                    (v.0 as f32) / 127.0,
                    (v.1 as f32) / 127.0,
                    (v.2 as f32) / 127.0,
                ]
            })
            .collect()
    }

    /// The coordinates used to map textures to this mesh.
    pub fn texture_coordinates(&self) -> Vec<[f32; 2]> {
        self.fragment
            .texture_coordinates
            .iter()
            .map(|v| [(v.0 as f32) / 256.0, (v.1 as f32) / 256.0])
            .collect()
    }

    /// Indices into the positions vector of this mesh. Expects that the mesh will be drawn as
    /// a triangle list.
    pub fn indices(&self) -> Vec<u32> {
        self.fragment
            .polygons
            .iter()
            .flat_map(|v| {
                vec![
                    v.vertex_indexes.0 as u32,
                    v.vertex_indexes.1 as u32,
                    v.vertex_indexes.2 as u32,
                ]
                .into_iter()
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Material<'a> {
    doc: &'a WldDoc<'a>,
    fragment: MaterialFragment,
    name: Option<&'a str>,
}

impl<'a> Material<'a> {
    /// The name of the material
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// The color texture for this material.
    pub fn base_color_texture(&self) -> Option<Texture> {
        self.doc
            .get(&self.fragment.reference)
            .and_then(|(_, texture_ref)| self.doc.get(&texture_ref.reference))
            .map(|(name, fragment)| Texture {
                doc: self.doc,
                fragment,
                name,
            })
    }
}

#[derive(Debug)]
pub struct Texture<'a> {
    doc: &'a WldDoc<'a>,
    fragment: TextureFragment,
    name: Option<&'a str>,
}

impl<'a> Texture<'a> {
    /// The name of the texture
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// The name of the source image used by this texture. Wld files in theory support multiple
    /// source images per texture but in practice only ever seem to have one.
    pub fn source(&self) -> Option<String> {
        self.fragment
            .references
            .iter()
            // [TextureFragment]s reference a [TextureImagesFragment]
            .map(|r| self.doc.get(&r))
            .flat_map(|image| match image {
                // The [TextureImagesFragment] itself contains a collection of filenames. In
                // practice this seems to always be just a single filename.
                Some((_, i)) => i
                    .entries
                    .iter()
                    // These also seem to be stored in all caps. The s3d files however store
                    // filenames in lowercase. This accounts for that.
                    .map(|e| e.file_name.to_lowercase())
                    .collect::<Vec<_>>(),
                None => vec![],
            })
            .nth(0)
    }
}
