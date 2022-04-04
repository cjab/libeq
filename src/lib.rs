///! # An Everquest .wld file loader
///! This is a work in progress but already loads enough data from wld files to be able to do some
///! basic rendering of models. The interface has been heavily influenced by the
///! [glTF](https://github.com/gltf-rs/gltf) crate. Parts of the wld file format are still not well
///! understood and future understanding may influence the api of this crate.
///!
///! # Examples
///! ```rust
///! let archive = eq_archive::read("gfaydark.s3d").unwrap();
///! let wld_data = archive.get("gfaydark.wld").unwrap();
///!
///! let wld = eq_wld::load(&wld_data).unwrap();
///!
///! // Iterate over meshes
///! for mesh in wld.meshes() {
///!     let name = mesh.name();
///!     let positions = mesh.positions();
///!     let normals = mesh.normals();
///!     let texture_coordinates = mesh.texture_coordinates();
///!     let indices = mesh.indices();
///!     let center = mesh.center();
///!     ...
///! }
///!
///! // Iterate over materials
///! for material in wld.materials() {
///!     let name = material.name();
///!     let texture = material.base_color_texture();
///!     let texture_source = texture.source();
///!     ...
///! }
///! ```
///!
///! # Acknowledgements
///! This project wouldn't have been possible without Windcatcher's [WLD File Reference](https://eqemu.gitbook.io/server/categories/zones/customizing-zones/wld-file-reference).
///! Some documentation has been reproduced as comments within the parser module. Names of file
///! fragments have been changed when another term from the [glTF reference](https://www.khronos.org/files/gltf20-reference-guide.pdf)
///! seemed like a better fit. The goal is that this will be usable in more modern engines and
///! hopefully the names used are more familiar in that context.
///!
pub mod parser;

use parser::{MaterialFragment, MeshFragment, MeshFragmentPolygonEntry, TextureFragment, WldDoc};
use std::error::Error;

pub struct WldError;

pub struct Wld(WldDoc);

/// Load and parse a wld file from a slice.
pub fn load(data: &[u8]) -> Result<Wld, Box<dyn Error>> {
    Ok(Wld::load(data))
}

impl Wld {
    // FIXME: Handle errors, do not panic!
    fn load(data: &[u8]) -> Wld {
        match WldDoc::parse(&data[..]) {
            Ok((_, wld_doc)) => Wld(wld_doc),
            Err(err) => panic!("Failed to parse Wld: {:?}", err),
        }
    }

    /// Iterate over all meshes in the wld file.
    pub fn meshes(&self) -> impl Iterator<Item = Mesh> + '_ {
        self.0.meshes().map(move |fragment| Mesh {
            doc: &self.0,
            fragment,
        })
    }

    /// Iterate over all materials in the wld file.
    pub fn materials(&self) -> impl Iterator<Item = Material> + '_ {
        self.0.materials().map(move |fragment| Material {
            doc: &self.0,
            fragment,
        })
    }
}

#[derive(Debug)]
pub struct Mesh<'a> {
    doc: &'a WldDoc,
    fragment: &'a MeshFragment,
}

impl<'a> Mesh<'a> {
    /// The name of the mesh
    pub fn name(&self) -> Option<&str> {
        self.doc.strings.get(self.fragment.name_reference)
    }

    pub fn center(&self) -> (f32, f32, f32) {
        let (x, z, y) = self.fragment.center;
        (x, y, z)
    }

    /// The positions of the vertices that make up this mesh.
    pub fn positions(&self) -> Vec<[f32; 3]> {
        let scale = 1.0 / (1 << self.fragment.scale) as f32;
        self.fragment
            .positions
            .iter()
            .map(|v| [v.0 as f32 * scale, v.2 as f32 * scale, v.1 as f32 * scale])
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
                    (v.2 as f32) / 127.0,
                    (v.1 as f32) / 127.0,
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

    /// A list of materials used by this mesh.
    pub fn materials(&self) -> Vec<Material> {
        let material_list = self
            .doc
            .get(&self.fragment.material_list_ref)
            .expect("Invalid material list reference");
        material_list
            .fragments
            .iter()
            .map(|fragment_ref| {
                self.doc
                    .get(&fragment_ref)
                    .expect("Invalid material reference")
            })
            .map(|fragment| Material {
                doc: &self.doc,
                fragment,
            })
            .collect()
    }

    /// Primitives belonging to this mesh.
    pub fn primitives(&self) -> Vec<Primitive> {
        let mut pos = 0;
        self.fragment
            .polygon_materials
            .iter()
            .enumerate()
            .map(|(index, (poly_count, ref material_idx))| {
                let count = *poly_count as usize;
                let next_pos = pos + count;
                let batch = pos..next_pos;
                pos = next_pos;
                Primitive {
                    mesh: self,
                    index,
                    fragments: &self
                        .fragment
                        .polygons
                        .get(batch)
                        .expect("Primitive fragments out of range"),
                    material_idx: *material_idx as usize,
                }
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Primitive<'a> {
    mesh: &'a Mesh<'a>,
    index: usize,
    fragments: &'a [MeshFragmentPolygonEntry],
    material_idx: usize,
}

impl<'a> Primitive<'a> {
    /// Indices of the positions making up this primitive. These refer to positions on the parent
    /// mesh.
    pub fn indices(&self) -> Vec<u32> {
        self.fragments
            .iter()
            .flat_map(|v| {
                vec![
                    v.vertex_indexes.0 as u32,
                    v.vertex_indexes.1 as u32,
                    v.vertex_indexes.2 as u32,
                ]
            })
            .collect()
    }

    pub fn positions(&self) -> Vec<[f32; 3]> {
        self.mesh.positions()
    }

    pub fn normals(&self) -> Vec<[f32; 3]> {
        self.mesh.normals()
    }

    pub fn texture_coordinates(&self) -> Vec<[f32; 2]> {
        self.mesh.texture_coordinates()
    }

    /// The material that this primitive uses.
    pub fn material(&self) -> Material {
        self.mesh.materials().remove(self.material_idx)
    }

    /// The index of this primitive in its parent mesh.
    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Debug)]
pub struct Material<'a> {
    doc: &'a WldDoc,
    fragment: &'a MaterialFragment,
}

impl<'a> Material<'a> {
    /// The name of the material
    pub fn name(&self) -> Option<&str> {
        self.doc.strings.get(self.fragment.name_reference)
    }

    /// The color texture for this material.
    pub fn base_color_texture(&self) -> Option<Texture> {
        self.doc
            .get(&self.fragment.reference)
            .and_then(|texture_ref| self.doc.get(&texture_ref.reference))
            .map(|fragment| Texture {
                doc: self.doc,
                fragment,
            })
    }
}

#[derive(Debug)]
pub struct Texture<'a> {
    doc: &'a WldDoc,
    fragment: &'a TextureFragment,
}

impl<'a> Texture<'a> {
    /// The name of the texture
    pub fn name(&self) -> Option<&str> {
        self.doc.strings.get(self.fragment.name_reference)
    }

    /// The name of the source image used by this texture. Wld files in theory support multiple
    /// source images per texture but in practice only ever seem to have one.
    pub fn source(&self) -> Option<String> {
        self.fragment
            .frame_references
            .iter()
            // [TextureFragment]s reference a [TextureImagesFragment]
            .map(|r| self.doc.get(&r))
            .flat_map(|image| match image {
                // The [TextureImagesFragment] itself contains a collection of filenames. In
                // practice this seems to always be just a single filename.
                Some(i) => i
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_meshes() {
        let wld_data = &include_bytes!("../fixtures/gfaydark.wld")[..];
        let wld = Wld::load(wld_data);
        let meshes = wld.meshes().collect::<Vec<_>>();

        assert_eq!(meshes.len(), 1597);
        assert_eq!(meshes[0].name(), Some("R8_DMSPRITEDEF"));
        assert_eq!(meshes[0].positions().len(), 8);
        assert_eq!(meshes[0].positions()[0], [0.0625, -0.09375, -36.0625]);
        assert_eq!(meshes[0].normals().len(), 8);
        assert_eq!(meshes[0].normals()[0], [0.22834645, 0.93700784, 0.24409449]);
        assert_eq!(meshes[0].texture_coordinates().len(), 8);
        assert_eq!(meshes[0].texture_coordinates()[0], [0.30078125, 0.30078125]);
        assert_eq!(meshes[0].indices().len(), 18);
        assert_eq!(meshes[0].indices()[0], 0);
        assert_eq!(meshes[0].center(), (-2502.0, 190.0, -2432.0));
    }

    #[test]
    fn it_builds_materials() {
        let wld_data = &include_bytes!("../fixtures/gfaydark.wld")[..];
        let wld = Wld::load(wld_data);
        let materials = wld.materials().collect::<Vec<_>>();

        assert_eq!(materials.len(), 33);
        assert_eq!(materials[0].name(), Some("SGRASS_MDF"));
        assert_eq!(
            materials[0].base_color_texture().unwrap().name(),
            Some("SGRASS_SPRITE")
        );
    }
}
