mod error;
pub mod fragments;
mod strings;

use core::fmt::Debug;
use std::collections::BTreeMap;

use itertools::{Either, Itertools};
use nom::bytes::complete::take;
pub use nom::error::{context, ErrorKind, VerboseError, VerboseErrorKind};
use nom::multi::count;
use nom::number::complete::le_u32;
use nom::IResult;
use nom::Offset;

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
    strings: StringHash,
    fragments: Vec<Box<FragmentType>>,
}

impl WldDoc {
    pub fn parse(input: &[u8]) -> Result<WldDoc, Vec<WldDocError>> {
        let (i, header) = WldHeader::parse(input).map_err(|e| vec![e.into()])?;

        let (i, string_hash_data) = take(header.string_hash_size)(i).map_err(|e| vec![e.into()])?;
        let strings = StringHash::new(string_hash_data);

        let (_i, fragment_headers) =
            count(FragmentHeader::parse, header.fragment_count as usize)(i)
                .map_err(|e| vec![e.into()])?;

        let (fragments, errors): (Vec<_>, Vec<_>) = fragment_headers
            .into_iter()
            .enumerate()
            .map(|(idx, h)| h.parse_body(idx))
            .partition_map(|res| match res {
                Ok(frag) => Either::Left(Box::new(frag)),
                Err(e) => Either::Right(e),
            });

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(WldDoc {
            header,
            strings,
            fragments,
        })
    }

    pub fn fragment_headers_by_offset(input: &[u8]) -> BTreeMap<usize, FragmentHeader> {
        let (i, header) = WldHeader::parse(input)
            .expect(&format!("{:?}", &input[..std::mem::size_of::<WldHeader>()]));
        let (_, i) = i.split_at(header.string_hash_size as usize);

        //let (i, _): (&[u8], &[u8]) =
        //    take::<u32, &[u8], nom::error::Error<&[u8]>>(header.string_hash_size)(i).unwrap();

        let mut fragment_headers = BTreeMap::new();
        let mut remaining = i;
        for idx in (0..header.fragment_count).into_iter() {
            let offset = input.len() - remaining.len();
            println!("Parsing fragment header {} at offset {:#10x}", idx, offset);

            let (x, fragment_header) = FragmentHeader::parse(remaining).expect(&format!(
                "Failed to parse fragment header {} at offset {:#10x}",
                idx, offset
            ));
            fragment_headers.insert(offset, fragment_header);
            remaining = x;
        }
        fragment_headers
    }

    pub fn dump_raw_fragments(input: &[u8]) -> WResult<Vec<FragmentHeader>> {
        let (i, header) = WldHeader::parse(input)?;
        let (i, _) = take(header.string_hash_size)(i)?;
        let (i, fragment_headers) =
            count(FragmentHeader::parse, header.fragment_count as usize)(i)?;

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
        self.fragments.get(idx).map(|f| f.as_ref())
    }

    /// Iterate over all fragments of a specific type
    pub fn fragment_iter<'a, T: 'static + Fragment>(&'a self) -> impl Iterator<Item = &'a T> + '_ {
        self.fragments
            .iter()
            .filter_map(|f| f.as_any().downcast_ref::<T>())
    }

    /// Iterate over all fragments
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Box<FragmentType>> + '_ {
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
        self.header.into_bytes()
    }

    pub fn strings_bytes(&self) -> Vec<u8> {
        self.strings.into_bytes()
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            self.header.into_bytes(),
            self.strings.into_bytes(),
            self.fragments
                .iter()
                .flat_map(|f| {
                    let mut field_data = f.into_bytes();
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
                    .into_bytes()
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
    pub fn parse(input: &[u8]) -> WResult<WldHeader> {
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

    pub fn into_bytes(&self) -> Vec<u8> {
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
    pub fn parse(input: &[u8]) -> WResult<FragmentHeader> {
        let (i, size) = context("size", le_u32)(input)?;
        let (i, fragment_type) = context("fragment_type", le_u32)(i)?;
        let (i, field_data) = context("field_data", take(size))(i)?;

        Ok((
            i,
            FragmentHeader {
                size,
                fragment_type,
                field_data,
            },
        ))
    }

    fn parse_body(self, index: usize) -> Result<FragmentType, WldDocError<'a>> {
        let parsed = match self.fragment_type {
            AlternateMeshFragment::TYPE_ID => Some(
                AlternateMeshFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::AlternateMesh(f.1))),
            ),
            BlitSpriteDefinitionFragment::TYPE_ID => Some(
                BlitSpriteDefinitionFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::BlitSpriteDefinition(f.1))),
            ),
            BlitSpriteReferenceFragment::TYPE_ID => Some(
                BlitSpriteReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::BlitSpriteReference(f.1))),
            ),
            VertexColorReferenceFragment::TYPE_ID => Some(
                VertexColorReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::VertexColorReference(f.1))),
            ),
            VertexColorFragment::TYPE_ID => Some(
                VertexColorFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::VertexColor(f.1))),
            ),
            MeshAnimatedVerticesFragment::TYPE_ID => Some(
                MeshAnimatedVerticesFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::MeshAnimatedVertices(f.1))),
            ),
            MeshAnimatedVerticesReferenceFragment::TYPE_ID => Some(
                MeshAnimatedVerticesReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::MeshAnimatedVerticesReference(f.1))),
            ),
            AmbientLightFragment::TYPE_ID => Some(
                AmbientLightFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::AmbientLight(f.1))),
            ),
            RegionFlagFragment::TYPE_ID => Some(
                RegionFlagFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::RegionFlag(f.1))),
            ),
            LightInfoFragment::TYPE_ID => Some(
                LightInfoFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::LightInfo(f.1))),
            ),
            LightSourceReferenceFragment::TYPE_ID => Some(
                LightSourceReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::LightSourceReference(f.1))),
            ),
            LightSourceFragment::TYPE_ID => Some(
                LightSourceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::LightSource(f.1))),
            ),
            PolygonAnimationReferenceFragment::TYPE_ID => Some(
                PolygonAnimationReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::PolygonAnimationReference(f.1))),
            ),
            PolygonAnimationFragment::TYPE_ID => Some(
                PolygonAnimationFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::PolygonAnimation(f.1))),
            ),
            FirstFragment::TYPE_ID => Some(
                FirstFragment::parse(&self.field_data).map(|f| (f.0, FragmentType::First(f.1))),
            ),
            ZoneUnknownFragment::TYPE_ID => Some(
                ZoneUnknownFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::ZoneUnknown(f.1))),
            ),
            SkeletonTrackSetReferenceFragment::TYPE_ID => Some(
                SkeletonTrackSetReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::SkeletonTrackSetReference(f.1))),
            ),
            CameraReferenceFragment::TYPE_ID => Some(
                CameraReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::CameraReference(f.1))),
            ),
            CameraFragment::TYPE_ID => Some(
                CameraFragment::parse(&self.field_data).map(|f| (f.0, FragmentType::Camera(f.1))),
            ),
            TwoDimensionalObjectReferenceFragment::TYPE_ID => Some(
                TwoDimensionalObjectReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::TwoDimensionalObjectReference(f.1))),
            ),
            TwoDimensionalObjectFragment::TYPE_ID => Some(
                TwoDimensionalObjectFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::TwoDimensionalObject(f.1))),
            ),
            ObjectLocationFragment::TYPE_ID => Some(
                ObjectLocationFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::ObjectLocation(f.1))),
            ),
            MobSkeletonPieceTrackReferenceFragment::TYPE_ID => Some(
                MobSkeletonPieceTrackReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::MobSkeletonPieceTrackReference(f.1))),
            ),
            MobSkeletonPieceTrackFragment::TYPE_ID => Some(
                MobSkeletonPieceTrackFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::MobSkeletonPieceTrack(f.1))),
            ),
            SkeletonTrackSetFragment::TYPE_ID => Some(
                SkeletonTrackSetFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::SkeletonTrackSet(f.1))),
            ),
            ModelFragment::TYPE_ID => Some(
                ModelFragment::parse(&self.field_data).map(|f| (f.0, FragmentType::Model(f.1))),
            ),
            BspTreeFragment::TYPE_ID => Some(
                BspTreeFragment::parse(&self.field_data).map(|f| (f.0, FragmentType::BspTree(f.1))),
            ),
            BspRegionFragment::TYPE_ID => Some(
                BspRegionFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::BspRegion(f.1))),
            ),
            MeshFragment::TYPE_ID => {
                Some(MeshFragment::parse(&self.field_data).map(|f| (f.0, FragmentType::Mesh(f.1))))
            }
            MaterialListFragment::TYPE_ID => Some(
                MaterialListFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::MaterialList(f.1))),
            ),
            MaterialFragment::TYPE_ID => Some(
                MaterialFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::Material(f.1))),
            ),
            TextureReferenceFragment::TYPE_ID => Some(
                TextureReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::TextureReference(f.1))),
            ),
            MeshReferenceFragment::TYPE_ID => Some(
                MeshReferenceFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::MeshReference(f.1))),
            ),
            TextureFragment::TYPE_ID => Some(
                TextureFragment::parse(&self.field_data).map(|f| (f.0, FragmentType::Texture(f.1))),
            ),
            TextureImagesFragment::TYPE_ID => Some(
                TextureImagesFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::TextureImages(f.1))),
            ),
            Unknown0x34Fragment::TYPE_ID => Some(
                Unknown0x34Fragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::Unknown0x34(f.1))),
            ),
            Unknown0x2eFragment::TYPE_ID => Some(
                Unknown0x2eFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::Unknown0x2e(f.1))),
            ),
            SphereListFragment::TYPE_ID => Some(
                SphereListFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::SphereList(f.1))),
            ),
            SphereListDefFragment::TYPE_ID => Some(
                SphereListDefFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::SphereListDef(f.1))),
            ),
            ParticleSpriteFragment::TYPE_ID => Some(
                ParticleSpriteFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::ParticleSprite(f.1))),
            ),
            ParticleSpriteDefFragment::TYPE_ID => Some(
                ParticleSpriteDefFragment::parse(&self.field_data)
                    .map(|f| (f.0, FragmentType::ParticleSpriteDef(f.1))),
            ),
            _ => None,
        };

        match parsed {
            Some(res) => res.map(|r| r.1).map_err(|e| match e.into() {
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

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.size.to_le_bytes()[..],
            &self.fragment_type.to_le_bytes()[..],
            &self.field_data,
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../fixtures/gfaydark.wld")[..];
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

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../fixtures/gfaydark.wld")[..];
        let wld_doc = WldDoc::parse(data).unwrap();

        let serialized_data = wld_doc.into_bytes();
        let deserialized_doc = WldDoc::parse(&serialized_data).unwrap();

        assert_eq!(wld_doc.header, deserialized_doc.header);
        assert_eq!(wld_doc.strings, deserialized_doc.strings);
        assert_eq!(wld_doc.fragments.len(), deserialized_doc.fragments.len());
        assert_eq!(
            wld_doc.fragments.first().unwrap().into_bytes(),
            deserialized_doc.fragments.first().unwrap().into_bytes()
        );
        assert_eq!(
            wld_doc.fragments.last().unwrap().into_bytes(),
            deserialized_doc.fragments.last().unwrap().into_bytes()
        );
    }
}
