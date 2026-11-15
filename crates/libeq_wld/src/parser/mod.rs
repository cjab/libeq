mod error;
pub mod fragments;
mod strings;
#[cfg(test)]
pub(crate) mod test_wld;

use core::fmt::Debug;
use std::collections::BTreeMap;

use itertools::{Either, Itertools};
use nom::IResult;
use nom::Offset;
use nom::Parser;
use nom::bytes::complete::take;
pub use nom::error::{ErrorKind, context};
use nom::multi::count;
use nom::number::complete::{le_i32, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use error::WldDocError;
pub use fragments::*;
pub use strings::{StringHash, StringReference};

pub type WResult<'a, O> = IResult<&'a [u8], O, WldDocError<'a>>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct WldDoc {
    header: WldHeader,
    format: WldFormat,
    strings: StringHash,
    fragments: Vec<FragmentType>,
}

/// The two known .wld layouts, told apart by the header's version.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WldFormat {
    /// Version `0x00015500`.
    Old,
    /// Version `0x1000C800`. Mesh ([DmSpriteDef2]) texture coordinates are 32-bit floats.
    New,
}

impl WldFormat {
    pub const OLD_VERSION: u32 = 0x0001_5500;
    pub const NEW_VERSION: u32 = 0x1000_C800;

    /// The layout a header version names, if it is a known one.
    pub fn from_version(version: u32) -> Option<Self> {
        match version {
            Self::OLD_VERSION => Some(Self::Old),
            Self::NEW_VERSION => Some(Self::New),
            _ => None,
        }
    }
}

/// Fragments are padded so their size is a multiple of 4. A parser that
/// stops more bytes than this short of a fragment's end has misread it.
const MAX_PADDING: usize = 3;

impl WldDoc {
    pub fn parse(input: &[u8]) -> Result<WldDoc, Vec<WldDocError<'_>>> {
        let (i, header) = WldHeader::parse(input).map_err(|e| vec![e.into()])?;
        let format = WldFormat::from_version(header.version).ok_or_else(|| {
            vec![WldDocError::UnsupportedVersion {
                version: header.version,
            }]
        })?;

        let (i, string_hash_data) = take(header.string_hash_size)
            .parse(i)
            .map_err(|e| vec![e.into()])?;
        let strings = StringHash::new(string_hash_data);

        let (_i, fragment_headers) = count(FragmentHeader::parse, header.fragment_count as usize)
            .parse(i)
            .map_err(|e| vec![e.into()])?;

        let (fragments, errors): (Vec<_>, Vec<_>) = fragment_headers
            .into_iter()
            .enumerate()
            .map(|(idx, h)| h.parse_body(idx, format))
            .partition_map(|res| match res {
                Ok(frag) => Either::Left(frag),
                Err(e) => Either::Right(e),
            });

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(WldDoc {
            header,
            format,
            strings,
            fragments,
        })
    }

    /// The layout of this file, from its header's version.
    pub fn format(&self) -> WldFormat {
        self.format
    }

    pub fn fragment_headers_by_offset(input: &[u8]) -> BTreeMap<usize, FragmentHeader<'_>> {
        let (i, header) = WldHeader::parse(input)
            .unwrap_or_else(|_| panic!("{:?}", &input[..std::mem::size_of::<WldHeader>()]));
        let (_, i) = i.split_at(header.string_hash_size as usize);

        //let (i, _): (&[u8], &[u8]) =
        //    take::<u32, &[u8], nom::error::Error<&[u8]>>(header.string_hash_size)(i).unwrap();

        let mut fragment_headers = BTreeMap::new();
        let mut remaining = i;
        for idx in 0..header.fragment_count {
            let offset = input.len() - remaining.len();
            println!("Parsing fragment header {} at offset {:#10x}", idx, offset);

            let (x, fragment_header) = FragmentHeader::parse(remaining).unwrap_or_else(|_| {
                panic!(
                    "Failed to parse fragment header {} at offset {:#10x}",
                    idx, offset
                )
            });
            fragment_headers.insert(offset, fragment_header);
            remaining = x;
        }
        fragment_headers
    }

    pub fn dump_raw_fragments(input: &[u8]) -> WResult<'_, Vec<FragmentHeader<'_>>> {
        let (i, header) = WldHeader::parse(input)?;
        let (i, _) = take(header.string_hash_size).parse(i)?;
        let (i, fragment_headers) =
            count(FragmentHeader::parse, header.fragment_count as usize).parse(i)?;

        Ok((i, fragment_headers))
    }

    /// Get a string given a string reference
    pub fn get_string(&self, string_reference: StringReference) -> Option<&str> {
        self.strings.get(string_reference)
    }

    /// Get a fragment given a fragment reference.
    pub fn get<T: 'static + Fragment>(&self, fragment_ref: &FragmentRef<T>) -> Option<&T> {
        match fragment_ref {
            FragmentRef::Name(_, _) => self.get_by_name_ref(fragment_ref),
            FragmentRef::Index(_, _) => self.get_by_index_ref(fragment_ref),
        }
    }

    /// Get a fragment given an index
    pub fn at(&self, idx: usize) -> Option<&FragmentType> {
        self.fragments.get(idx)
    }

    /// Iterate over all fragments of a specific type
    pub fn fragment_iter<'a, T: 'static + Fragment>(&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self.fragments
            .iter()
            .filter_map(|f| f.as_any().downcast_ref::<T>())
    }

    /// Iterate over all fragments
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a FragmentType> + 'a {
        self.fragments.iter()
    }

    fn get_by_index_ref<T: 'static + Fragment>(&self, fragment_ref: &FragmentRef<T>) -> Option<&T> {
        let idx = if let FragmentRef::Index(idx, _) = fragment_ref {
            idx
        } else {
            return None;
        };

        self.fragments
            .get((idx - 1) as usize)?
            .as_any()
            .downcast_ref()
    }

    fn get_by_name_ref<T: 'static + Fragment>(&self, fragment_ref: &FragmentRef<T>) -> Option<&T> {
        let name_ref = if let FragmentRef::Name(name_ref, _) = fragment_ref {
            *name_ref
        } else {
            return None;
        };

        if let Some(target_name) = self.strings.get(name_ref) {
            self.fragments
                .iter()
                .find(|f| self.strings.get(*f.name_ref()) == Some(target_name))?
                .as_any()
                .downcast_ref()
        } else {
            None
        }
    }

    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn header_bytes(&self) -> Vec<u8> {
        self.header.to_bytes()
    }

    pub fn strings_bytes(&self) -> Vec<u8> {
        self.strings.to_bytes()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.header.to_bytes(),
            self.strings.to_bytes(),
            self.fragments
                .iter()
                .flat_map(|f| {
                    let mut field_data = f.to_bytes();
                    let size = field_data.len();
                    // Field data must be padded so that it aligns on 4 bytes
                    if (size % 4) > 0 {
                        field_data.resize(size + (4 - (size % 4)), 0);
                    }
                    FragmentHeader {
                        size: field_data.len() as u32,
                        fragment_type: f.type_id(),
                        field_data: &field_data[..],
                    }
                    .to_bytes()
                })
                .collect(),
            // What is this?
            vec![0xff, 0xff, 0xff, 0xff],
        ]
        .concat()
    }
}

/// This header is present at the beginning of every .wld file.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct WldHeader {
    /// The file signature that signals that this is a .wld file.
    magic: u32,

    /// Two known versions of .wld file exist.
    /// * The old format - `0x00015500`
    /// * The new format - `0x1000C800`
    version: u32,

    /// The number of fragments in the .wld file minus 1
    fragment_count: u32,

    /// The number of region fragments in the file
    region_count: u32,

    /// The size of the largest fragment in the file
    max_object_bytes: u32,

    /// The size of the string hash in bytes.
    string_hash_size: u32,

    /// The number of strings in the string hash
    string_count: u32,
}

impl WldHeader {
    pub fn parse(input: &[u8]) -> WResult<'_, WldHeader> {
        let (i, magic) = le_u32(input)?;
        let (i, version) = le_u32(i)?;
        let (i, fragment_count) = le_u32(i)?;
        let (i, region_count) = le_u32(i)?;
        let (i, max_object_bytes) = le_u32(i)?;
        let (i, string_hash_size) = le_u32(i)?;
        let (i, string_count) = le_u32(i)?;

        Ok((
            i,
            WldHeader {
                magic,
                version,
                fragment_count,
                region_count,
                max_object_bytes,
                string_hash_size,
                string_count,
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &self.magic.to_le_bytes()[..],
            &self.version.to_le_bytes()[..],
            &self.fragment_count.to_le_bytes()[..],
            &self.region_count.to_le_bytes()[..],
            &self.max_object_bytes.to_le_bytes()[..],
            &self.string_hash_size.to_le_bytes()[..],
            &self.string_count.to_le_bytes()[..],
        ]
        .concat()
    }
}

type FragmentTypeId = u32;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// All fragments begin with the following header
pub struct FragmentHeader<'a> {
    /// The size of the fragment in bytes. All fragments are padded such that `size`
    /// is evenly divisible by 4 and Size should reflect the padded value.
    pub size: u32,

    /// The fragment type. This will typically be a value in the
    /// range 0x03 to 0x37 and tells the file reader which specific kind of fragment
    /// it is. Some fragment types are plain fragments and some are reference
    /// fragments, the type determines which.
    pub fragment_type: FragmentTypeId,

    /// Each fragment may have a string name, stored in encoded form in the .wld
    /// file string hash. `name_reference` provides a way to retrieve that name.
    /// If the fragment has a string name, `name_reference` will contain the
    /// negative value of the string’s index in the string hash.
    ///
    /// For example, if the string is at position 31 in the string hash, then
    /// `name_reference` should contain –31. Values greater than 0 mean that the
    /// fragment doesn’t have a string name. Effectively, a value of 0 also means
    /// that the fragment doesn’t have a string name, and the first byte in the string
    /// hash is always preallocated to reflect this (it’s a null character that is
    /// encoded along with everything else).
    ///
    /// All fragments without a name will have a `name_reference` of 0.
    /// The one exception being the 0x35 fragment which will always reference 0xFF000000.
    pub field_data: &'a [u8],
}

impl<'a> FragmentHeader<'a> {
    pub fn parse(input: &[u8]) -> WResult<'_, FragmentHeader<'_>> {
        let (i, size) = context("size", le_u32).parse(input)?;
        let (i, fragment_type) = context("fragment_type", le_u32).parse(i)?;
        let (i, field_data) = context("field_data", take(size)).parse(i)?;

        Ok((
            i,
            FragmentHeader {
                size,
                fragment_type,
                field_data,
            },
        ))
    }

    /// Parses the fragment body. A body its parser does not read to its end,
    /// beyond alignment padding, is an error.
    fn parse_body(self, index: usize, format: WldFormat) -> Result<FragmentType, WldDocError<'a>> {
        let parsed = match self.fragment_type {
            DmSpriteDef::TYPE_ID => match self.detect_0x2c_variant() {
                FragmentGame::EverQuest => Some(
                    DmSpriteDef::parse(self.field_data)
                        .map(|f| (f.0, FragmentType::DmSpriteDef(f.1))),
                ),
                FragmentGame::ReturnToKrondor => Some(
                    BmInfoRtk::parse(self.field_data).map(|f| (f.0, FragmentType::BmInfoRtk(f.1))),
                ),
                FragmentGame::Tanarus => Some(
                    WorldVertices::parse(self.field_data)
                        .map(|f| (f.0, FragmentType::WorldVertices(f.1))),
                ),
            },
            BlitSpriteDef::TYPE_ID => Some(
                BlitSpriteDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::BlitSpriteDef(f.1))),
            ),
            BlitSprite::TYPE_ID => Some(
                BlitSprite::parse(self.field_data).map(|f| (f.0, FragmentType::BlitSprite(f.1))),
            ),
            DmRGBTrack::TYPE_ID => Some(
                DmRGBTrack::parse(self.field_data).map(|f| (f.0, FragmentType::DmRGBTrack(f.1))),
            ),
            DmRGBTrackDef::TYPE_ID => Some(
                DmRGBTrackDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::DmRGBTrackDef(f.1))),
            ),
            DmTrackDef2::TYPE_ID => Some(
                DmTrackDef2::parse(self.field_data).map(|f| (f.0, FragmentType::DmTrackDef2(f.1))),
            ),
            DmTrack::TYPE_ID => {
                Some(DmTrack::parse(self.field_data).map(|f| (f.0, FragmentType::DmTrack(f.1))))
            }
            AmbientLight::TYPE_ID => Some(
                AmbientLight::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::AmbientLight(f.1))),
            ),
            Zone::TYPE_ID => {
                Some(Zone::parse(self.field_data).map(|f| (f.0, FragmentType::Zone(f.1))))
            }
            PointLight::TYPE_ID => Some(
                PointLight::parse(self.field_data).map(|f| (f.0, FragmentType::PointLight(f.1))),
            ),
            Light::TYPE_ID => {
                Some(Light::parse(self.field_data).map(|f| (f.0, FragmentType::Light(f.1))))
            }
            LightDef::TYPE_ID => {
                Some(LightDef::parse(self.field_data).map(|f| (f.0, FragmentType::LightDef(f.1))))
            }
            Polyhedron::TYPE_ID => Some(
                Polyhedron::parse(self.field_data).map(|f| (f.0, FragmentType::Polyhedron(f.1))),
            ),
            PolyhedronDef::TYPE_ID => Some(
                PolyhedronDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::PolyhedronDef(f.1))),
            ),
            GlobalAmbientLightDef::TYPE_ID => Some(
                GlobalAmbientLightDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::GlobalAmbientLightDef(f.1))),
            ),
            Sphere::TYPE_ID => {
                Some(Sphere::parse(self.field_data).map(|f| (f.0, FragmentType::Sphere(f.1))))
            }
            HierarchicalSprite::TYPE_ID => Some(
                HierarchicalSprite::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::HierarchicalSprite(f.1))),
            ),
            Sprite3D::TYPE_ID => {
                Some(Sprite3D::parse(self.field_data).map(|f| (f.0, FragmentType::Sprite3D(f.1))))
            }
            Sprite3DDef::TYPE_ID => Some(
                Sprite3DDef::parse(self.field_data).map(|f| (f.0, FragmentType::Sprite3DDef(f.1))),
            ),
            Sprite2D::TYPE_ID => {
                Some(Sprite2D::parse(self.field_data).map(|f| (f.0, FragmentType::Sprite2D(f.1))))
            }
            Sprite2DDef::TYPE_ID => Some(
                Sprite2DDef::parse(self.field_data).map(|f| (f.0, FragmentType::Sprite2DDef(f.1))),
            ),
            Actor::TYPE_ID => {
                Some(Actor::parse(self.field_data).map(|f| (f.0, FragmentType::Actor(f.1))))
            }
            Track::TYPE_ID => {
                Some(Track::parse(self.field_data).map(|f| (f.0, FragmentType::Track(f.1))))
            }
            TrackDef::TYPE_ID => {
                Some(TrackDef::parse(self.field_data).map(|f| (f.0, FragmentType::TrackDef(f.1))))
            }
            HierarchicalSpriteDef::TYPE_ID => Some(
                HierarchicalSpriteDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::HierarchicalSpriteDef(f.1))),
            ),
            ActorDef::TYPE_ID => {
                Some(ActorDef::parse(self.field_data).map(|f| (f.0, FragmentType::ActorDef(f.1))))
            }
            WorldTree::TYPE_ID => {
                Some(WorldTree::parse(self.field_data).map(|f| (f.0, FragmentType::WorldTree(f.1))))
            }
            Region::TYPE_ID => {
                Some(Region::parse(self.field_data).map(|f| (f.0, FragmentType::Region(f.1))))
            }
            DmSpriteDef2::TYPE_ID => Some(
                DmSpriteDef2::parse_format(self.field_data, format)
                    .map(|f| (f.0, FragmentType::DmSpriteDef2(f.1))),
            ),
            MaterialPalette::TYPE_ID => Some(
                MaterialPalette::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::MaterialPalette(f.1))),
            ),
            MaterialDef::TYPE_ID => Some(
                MaterialDef::parse(self.field_data).map(|f| (f.0, FragmentType::MaterialDef(f.1))),
            ),
            SimpleSprite::TYPE_ID => Some(
                SimpleSprite::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::SimpleSprite(f.1))),
            ),
            DmSprite::TYPE_ID => {
                Some(DmSprite::parse(self.field_data).map(|f| (f.0, FragmentType::DmSprite(f.1))))
            }
            SimpleSpriteDef::TYPE_ID => Some(
                SimpleSpriteDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::SimpleSpriteDef(f.1))),
            ),
            BmInfo::TYPE_ID => {
                Some(BmInfo::parse(self.field_data).map(|f| (f.0, FragmentType::BmInfo(f.1))))
            }
            ParticleCloudDef::TYPE_ID => Some(
                ParticleCloudDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::ParticleCloudDef(f.1))),
            ),
            DmTrackDef::TYPE_ID => Some(
                DmTrackDef::parse(self.field_data).map(|f| (f.0, FragmentType::DmTrackDef(f.1))),
            ),
            SphereList::TYPE_ID => Some(
                SphereList::parse(self.field_data).map(|f| (f.0, FragmentType::SphereList(f.1))),
            ),
            SphereListDef::TYPE_ID => Some(
                SphereListDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::SphereListDef(f.1))),
            ),
            ParticleSprite::TYPE_ID => Some(
                ParticleSprite::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::ParticleSprite(f.1))),
            ),
            ParticleSpriteDef::TYPE_ID => Some(
                ParticleSpriteDef::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::ParticleSpriteDef(f.1))),
            ),
            DefaultPaletteFile::TYPE_ID => Some(
                DefaultPaletteFile::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::DefaultPaletteFile(f.1))),
            ),
            Sprite4D::TYPE_ID => {
                Some(Sprite4D::parse(self.field_data).map(|f| (f.0, FragmentType::Sprite4D(f.1))))
            }
            Sprite4DDef::TYPE_ID => Some(
                Sprite4DDef::parse(self.field_data).map(|f| (f.0, FragmentType::Sprite4DDef(f.1))),
            ),
            DirectionalLight::TYPE_ID => Some(
                DirectionalLight::parse(self.field_data)
                    .map(|f| (f.0, FragmentType::DirectionalLight(f.1))),
            ),
            _ => None,
        };

        match parsed {
            Some(Ok((rest, _))) if rest.len() > MAX_PADDING => Err(WldDocError::UnreadBytes {
                index,
                bytes: rest.len(),
                header: self,
            }),
            Some(Ok((_, fragment))) => Ok(fragment),
            Some(Err(e)) => Err(match e.into() {
                WldDocError::Parse { input, message } => WldDocError::ParseFragment {
                    index,
                    offset: self.field_data.offset(input),
                    header: self,
                    message,
                },
                // This should never happen, the parse functions _only_
                // generate WldDocError::Parse errors
                e => panic!(
                    "Invalid error type encountered when parsing fragment body: {:?}",
                    e
                ),
            }),
            None => Err(WldDocError::UnknownFragment {
                index,
                header: self,
            }),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &self.size.to_le_bytes()[..],
            &self.fragment_type.to_le_bytes()[..],
            self.field_data,
        ]
        .concat()
    }

    /// Each game appears to have it's own custom 0x2c
    ///   EQ 0x2c starts with name ref (negative int)
    ///   Tanarus 0x2c starts with the vertex count (positive int)
    ///   RtK 0x2c is very small, 32 bytes was the largest I could find
    fn detect_0x2c_variant(&self) -> FragmentGame {
        if self.size < 50 {
            return FragmentGame::ReturnToKrondor;
        }

        match le_i32::<_, nom::error::Error<&[u8]>>(self.field_data) {
            Ok((_, n)) if n > 0 => FragmentGame::Tanarus,
            _ => FragmentGame::EverQuest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_wld::{Coordinates, WldBuilder, one_triangle, palette};
    use super::*;

    fn only_mesh(doc: &WldDoc) -> &DmSpriteDef2 {
        let meshes: Vec<_> = doc.fragment_iter::<DmSpriteDef2>().collect();
        assert_eq!(meshes.len(), 1);
        meshes[0]
    }

    #[test]
    fn it_reads_new_format_mesh_coordinates_as_floats() {
        let data = one_triangle(
            WldFormat::NEW_VERSION,
            0x8000_0001,
            Coordinates::New([(0.0, 0.0), (1.0, 0.0), (0.5, -2.25)]),
        );
        let doc = WldDoc::parse(&data).unwrap();
        let mesh = only_mesh(&doc);

        assert_eq!(doc.format(), WldFormat::New);
        assert_eq!(
            mesh.texture_coordinates,
            TextureCoordinates::New(vec![(0.0, 0.0), (1.0, 0.0), (0.5, -2.25)])
        );
        // What follows the coordinates is read where the format puts it.
        assert_eq!(mesh.vertex_normals, vec![(0, 0, 127); 3]);
        assert_eq!(mesh.faces[0].vertex_indexes, (0, 1, 2));
        assert_eq!(mesh.face_material_groups, vec![(1, 0)]);
        assert_eq!(mesh.vertex_material_groups, vec![(3, 0)]);
    }

    #[test]
    fn it_reads_old_format_mesh_coordinates_as_integers() {
        let data = one_triangle(
            WldFormat::OLD_VERSION,
            0x8000_0001,
            Coordinates::Old([(0, 0), (256, 0), (128, -576)]),
        );
        let doc = WldDoc::parse(&data).unwrap();
        let mesh = only_mesh(&doc);

        assert_eq!(doc.format(), WldFormat::Old);
        assert_eq!(
            mesh.texture_coordinates,
            TextureCoordinates::Old(vec![(0, 0), (256, 0), (128, -576)])
        );
        assert_eq!(mesh.faces[0].vertex_indexes, (0, 1, 2));
        assert_eq!(mesh.face_material_groups, vec![(1, 0)]);
    }

    #[test]
    fn it_parses_materials_with_unlisted_user_defined_types() {
        let data = one_triangle(
            WldFormat::OLD_VERSION,
            0x8000_0018,
            Coordinates::Old([(0, 0), (256, 0), (128, 0)]),
        );
        let doc = WldDoc::parse(&data).unwrap();
        let materials: Vec<_> = doc.fragment_iter::<MaterialDef>().collect();

        assert_eq!(
            materials[0].render_method,
            RenderMethod::UserDefined {
                material_type: MaterialType::Unlisted(0x18)
            }
        );
        let reparsed = WldDoc::parse(&doc.to_bytes()).unwrap();
        assert_eq!(reparsed.to_bytes(), doc.to_bytes());
    }

    #[test]
    fn it_parses_vertex_animation_with_no_padding() {
        let mut wld = WldBuilder::new(WldFormat::OLD_VERSION);
        let name = wld.name("T_DMTRACKDEF");
        // 18 + 6 × 1 vertex × 1 frame = 24 bytes, already a multiple of 4.
        let mut body = Vec::new();
        body.extend(name.to_le_bytes());
        body.extend(0u32.to_le_bytes());
        for word in [1u16, 1, 100, 0, 9, 5, 6, 7] {
            body.extend(word.to_le_bytes());
        }
        wld.fragment(DmTrackDef2::TYPE_ID, body);
        let doc = WldDoc::parse(&wld.build()).unwrap();
        let tracks: Vec<_> = doc.fragment_iter::<DmTrackDef2>().collect();

        assert_eq!(tracks[0].frames, vec![vec![(5, 6, 7)]]);
        assert_eq!(tracks[0].size6, None);
    }

    #[test]
    fn it_round_trips_both_formats() {
        for data in [
            one_triangle(
                WldFormat::NEW_VERSION,
                0x8000_0001,
                Coordinates::New([(0.0, 0.0), (1.0, 0.0), (0.5, -2.25)]),
            ),
            one_triangle(
                WldFormat::OLD_VERSION,
                0x8000_0001,
                Coordinates::Old([(0, 0), (256, 0), (128, -576)]),
            ),
        ] {
            let doc = WldDoc::parse(&data).unwrap();
            let reparsed = WldDoc::parse(&doc.to_bytes()).unwrap();
            assert_eq!(reparsed.format(), doc.format());
            assert_eq!(only_mesh(&reparsed), only_mesh(&doc));
            assert_eq!(reparsed.to_bytes(), doc.to_bytes());
        }
    }

    #[test]
    fn it_rejects_unknown_versions_by_name() {
        let data = one_triangle(
            0x1234_5678,
            0x8000_0001,
            Coordinates::Old([(0, 0), (256, 0), (128, 0)]),
        );
        let errors = WldDoc::parse(&data).unwrap_err();

        assert!(matches!(
            errors[..],
            [WldDocError::UnsupportedVersion {
                version: 0x1234_5678
            }]
        ));
    }

    #[test]
    fn it_rejects_fragments_not_read_to_their_end() {
        let mut wld = WldBuilder::new(WldFormat::OLD_VERSION);
        let name = wld.name("P_MP");
        // The meshes in the tests above end with a byte of padding, which
        // parses; these eight bytes are data nobody read.
        let mut overlong = palette(name, &[]);
        overlong.extend([0xab; 8]);
        wld.fragment(MaterialPalette::TYPE_ID, palette(name, &[]));
        wld.fragment(MaterialPalette::TYPE_ID, overlong);
        let data = wld.build();
        let errors = WldDoc::parse(&data).unwrap_err();

        let [
            WldDocError::UnreadBytes {
                index: 1,
                bytes: 8,
                header,
            },
        ] = &errors[..]
        else {
            panic!("expected one unread-bytes error, got {errors:?}");
        };
        assert_eq!(header.fragment_type, MaterialPalette::TYPE_ID);
    }

    #[test]
    fn it_accepts_up_to_three_bytes_of_padding() {
        for padding in 0..=3 {
            let mut wld = WldBuilder::new(WldFormat::OLD_VERSION);
            let name = wld.name("P_MP");
            let mut body = palette(name, &[]);
            body.extend(vec![0; padding]);
            wld.unpadded_fragment(MaterialPalette::TYPE_ID, body);
            assert!(
                WldDoc::parse(&wld.build()).is_ok(),
                "{padding} bytes of padding should parse"
            );
        }
    }

    #[cfg(feature = "gamedata")]
    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../fixtures/gamedata/gfaydark.wld")[..];
        let wld_doc = WldDoc::parse(data).unwrap();
        assert_eq!(wld_doc.header.magic, 1414544642);
        assert_eq!(wld_doc.header.version, 0x00015500);
        assert_eq!(wld_doc.header.fragment_count, 4646);
        assert_eq!(wld_doc.header.region_count, 2905);
        assert_eq!(wld_doc.header.max_object_bytes, 162660);
        assert_eq!(wld_doc.header.string_hash_size, 52692);
        assert_eq!(wld_doc.header.string_count, 4609);
        assert_eq!(wld_doc.fragments.len(), 4646);
        assert_eq!(wld_doc.strings.get(StringReference::new(0)), Some(""));
        assert_eq!(wld_doc.strings.get(StringReference::new(1)), Some("SGRASS"));
        assert_eq!(wld_doc.strings.get(StringReference::new(2)), None);
    }

    #[cfg(feature = "gamedata")]
    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../fixtures/gamedata/gfaydark.wld")[..];
        let wld_doc = WldDoc::parse(data).unwrap();

        let serialized_data = wld_doc.to_bytes();
        let deserialized_doc = WldDoc::parse(&serialized_data).unwrap();

        assert_eq!(wld_doc.header, deserialized_doc.header);
        assert_eq!(wld_doc.strings, deserialized_doc.strings);
        assert_eq!(wld_doc.fragments.len(), deserialized_doc.fragments.len());
        assert_eq!(
            wld_doc.fragments.first().unwrap().to_bytes(),
            deserialized_doc.fragments.first().unwrap().to_bytes()
        );
        assert_eq!(
            wld_doc.fragments.last().unwrap().to_bytes(),
            deserialized_doc.fragments.last().unwrap().to_bytes()
        );
    }

    #[test]
    fn it_detects_eq_0x2c() {
        // EverQuest 0x2c starts with name_reference (negative i32)
        // Creating synthetic data: first 4 bytes = -44 (0xFFFFFFD4) as i32
        // followed by enough padding to make size >= 50
        let mut data = vec![0u8; 60];
        let name_ref: i32 = -44;
        data[0..4].copy_from_slice(&name_ref.to_le_bytes());

        let header = FragmentHeader {
            size: data.len() as u32,
            fragment_type: 0x2c,
            field_data: &data,
        };

        assert_eq!(header.detect_0x2c_variant(), FragmentGame::EverQuest);
    }

    #[test]
    fn it_detects_tanarus_0x2c() {
        // Tanarus 0x2c starts with vertex_count (positive i32)
        // Creating synthetic data: first 4 bytes = 100 (positive)
        // followed by enough padding to make size >= 50
        let mut data = vec![0u8; 60];
        let vertex_count: i32 = 100;
        data[0..4].copy_from_slice(&vertex_count.to_le_bytes());

        let header = FragmentHeader {
            size: data.len() as u32,
            fragment_type: 0x2c,
            field_data: &data,
        };

        assert_eq!(header.detect_0x2c_variant(), FragmentGame::Tanarus);
    }

    #[test]
    fn it_detects_rtk_0x2c() {
        // RTK 0x2c is very small (< 50 bytes)
        // Creating synthetic data: just 32 bytes
        let data = vec![0u8; 32];

        let header = FragmentHeader {
            size: data.len() as u32,
            fragment_type: 0x2c,
            field_data: &data,
        };

        assert_eq!(header.detect_0x2c_variant(), FragmentGame::ReturnToKrondor);
    }
}
