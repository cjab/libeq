pub mod fragments;
mod strings;

use core::fmt::{Debug, Error, Formatter};

use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use fragments::*;
pub use strings::{StringHash, StringReference};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct WldDoc {
    header: WldHeader,
    strings: StringHash,
    fragments: Vec<Box<FragmentType>>,
}

impl Debug for dyn Fragment + 'static {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), Error> {
        Ok(())
    }
}

impl WldDoc {
    pub fn parse(input: &[u8]) -> IResult<&[u8], WldDoc> {
        let (i, header) = WldHeader::parse(input)?;
        let (remaining, (string_hash_data, fragment_headers)) = tuple((
            take(header.string_hash_size),
            count(FragmentHeader::parse, header.fragment_count as usize),
        ))(i)?;
        let strings = StringHash::new(string_hash_data);
        let fragments = fragment_headers.iter().map(|h| h.parse_body()).collect();

        Ok((
            remaining,
            WldDoc {
                header,
                strings,
                fragments,
            },
        ))
    }

    pub fn dump_raw_fragments(input: &[u8]) -> IResult<&[u8], Vec<FragmentHeader>> {
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
            self.fragments.iter().flat_map(|f| f.into_bytes()).collect(),
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

    /// Believed to contain the number of 0x22 BSP region fragments in the file
    header_3: u32,

    /// _Unknown_ - Usually contains `0x000680D4`.
    header_4: u32,

    /// The size of the string hash in bytes.
    string_hash_size: u32,

    /// _Unknown_ - Possibly contains the number of fragments in the file minus the
    /// number of 0x03 fragments, minus 6
    header_6: u32,
}

impl WldHeader {
    pub fn parse(input: &[u8]) -> IResult<&[u8], WldHeader> {
        let (
            remaining,
            (magic, version, fragment_count, header_3, header_4, string_hash_size, header_6),
        ) = tuple((le_u32, le_u32, le_u32, le_u32, le_u32, le_u32, le_u32))(input)?;
        Ok((
            remaining,
            WldHeader {
                magic,
                version,
                fragment_count,
                header_3,
                header_4,
                string_hash_size,
                header_6,
            },
        ))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.magic.to_le_bytes()[..],
            &self.version.to_le_bytes()[..],
            &self.fragment_count.to_le_bytes()[..],
            &self.header_3.to_le_bytes()[..],
            &self.header_4.to_le_bytes()[..],
            &self.string_hash_size.to_le_bytes()[..],
            &self.header_6.to_le_bytes()[..],
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
    pub fn parse(input: &[u8]) -> IResult<&[u8], FragmentHeader> {
        let (i, (size, fragment_type)) = tuple((le_u32, le_u32))(input)?;
        let (remaining, field_data) = take(size)(i)?;

        Ok((
            remaining,
            FragmentHeader {
                size,
                fragment_type,
                field_data,
            },
        ))
    }

    fn parse_body(&self) -> Box<FragmentType> {
        match self.fragment_type {
            AlternateMeshFragment::TYPE_ID => Box::new(FragmentType::AlternateMesh(
                AlternateMeshFragment::parse(&self.field_data).unwrap().1,
            )),
            VertexColorReferenceFragment::TYPE_ID => Box::new(FragmentType::VertexColorReference(
                VertexColorReferenceFragment::parse(&self.field_data)
                    .unwrap()
                    .1,
            )),
            VertexColorFragment::TYPE_ID => Box::new(FragmentType::VertexColor(
                VertexColorFragment::parse(&self.field_data).unwrap().1,
            )),
            MeshAnimatedVerticesFragment::TYPE_ID => Box::new(FragmentType::MeshAnimatedVertices(
                MeshAnimatedVerticesFragment::parse(&self.field_data)
                    .unwrap()
                    .1,
            )),
            MeshAnimatedVerticesReferenceFragment::TYPE_ID => {
                Box::new(FragmentType::MeshAnimatedVerticesReference(
                    MeshAnimatedVerticesReferenceFragment::parse(&self.field_data)
                        .unwrap()
                        .1,
                ))
            }
            AmbientLightFragment::TYPE_ID => Box::new(FragmentType::AmbientLight(
                AmbientLightFragment::parse(&self.field_data).unwrap().1,
            )),
            RegionFlagFragment::TYPE_ID => Box::new(FragmentType::RegionFlag(
                RegionFlagFragment::parse(&self.field_data).unwrap().1,
            )),
            LightInfoFragment::TYPE_ID => Box::new(FragmentType::LightInfo(
                LightInfoFragment::parse(&self.field_data).unwrap().1,
            )),
            LightSourceReferenceFragment::TYPE_ID => Box::new(FragmentType::LightSourceReference(
                LightSourceReferenceFragment::parse(&self.field_data)
                    .unwrap()
                    .1,
            )),
            LightSourceFragment::TYPE_ID => Box::new(FragmentType::LightSource(
                LightSourceFragment::parse(&self.field_data).unwrap().1,
            )),
            PolygonAnimationReferenceFragment::TYPE_ID => {
                Box::new(FragmentType::PolygonAnimationReference(
                    PolygonAnimationReferenceFragment::parse(&self.field_data)
                        .unwrap()
                        .1,
                ))
            }
            PolygonAnimationFragment::TYPE_ID => Box::new(FragmentType::PolygonAnimation(
                PolygonAnimationFragment::parse(&self.field_data).unwrap().1,
            )),
            FirstFragment::TYPE_ID => Box::new(FragmentType::First(
                FirstFragment::parse(&self.field_data).unwrap().1,
            )),
            ZoneUnknownFragment::TYPE_ID => Box::new(FragmentType::ZoneUnknown(
                ZoneUnknownFragment::parse(&self.field_data).unwrap().1,
            )),
            SkeletonTrackSetReferenceFragment::TYPE_ID => {
                Box::new(FragmentType::SkeletonTrackSetReference(
                    SkeletonTrackSetReferenceFragment::parse(&self.field_data)
                        .unwrap()
                        .1,
                ))
            }
            CameraReferenceFragment::TYPE_ID => Box::new(FragmentType::CameraReference(
                CameraReferenceFragment::parse(&self.field_data).unwrap().1,
            )),
            CameraFragment::TYPE_ID => Box::new(FragmentType::Camera(
                CameraFragment::parse(&self.field_data).unwrap().1,
            )),
            TwoDimensionalObjectReferenceFragment::TYPE_ID => {
                Box::new(FragmentType::TwoDimensionalObjectReference(
                    TwoDimensionalObjectReferenceFragment::parse(&self.field_data)
                        .unwrap()
                        .1,
                ))
            }
            TwoDimensionalObjectFragment::TYPE_ID => Box::new(FragmentType::TwoDimensionalObject(
                TwoDimensionalObjectFragment::parse(&self.field_data)
                    .unwrap()
                    .1,
            )),
            ObjectLocationFragment::TYPE_ID => Box::new(FragmentType::ObjectLocation(
                ObjectLocationFragment::parse(&self.field_data).unwrap().1,
            )),
            MobSkeletonPieceTrackReferenceFragment::TYPE_ID => {
                Box::new(FragmentType::MobSkeletonPieceTrackReference(
                    MobSkeletonPieceTrackReferenceFragment::parse(&self.field_data)
                        .unwrap()
                        .1,
                ))
            }
            MobSkeletonPieceTrackFragment::TYPE_ID => {
                Box::new(FragmentType::MobSkeletonPieceTrack(
                    MobSkeletonPieceTrackFragment::parse(&self.field_data)
                        .unwrap()
                        .1,
                ))
            }
            SkeletonTrackSetFragment::TYPE_ID => Box::new(FragmentType::SkeletonTrackSet(
                SkeletonTrackSetFragment::parse(&self.field_data).unwrap().1,
            )),
            ModelFragment::TYPE_ID => Box::new(FragmentType::Model(
                ModelFragment::parse(&self.field_data).unwrap().1,
            )),
            BspTreeFragment::TYPE_ID => Box::new(FragmentType::BspTree(
                BspTreeFragment::parse(&self.field_data).unwrap().1,
            )),
            BspRegionFragment::TYPE_ID => Box::new(FragmentType::BspRegion(
                BspRegionFragment::parse(&self.field_data).unwrap().1,
            )),
            MeshFragment::TYPE_ID => Box::new(FragmentType::Mesh(
                MeshFragment::parse(&self.field_data).unwrap().1,
            )),
            MaterialListFragment::TYPE_ID => Box::new(FragmentType::MaterialList(
                MaterialListFragment::parse(&self.field_data).unwrap().1,
            )),
            MaterialFragment::TYPE_ID => Box::new(FragmentType::Material(
                MaterialFragment::parse(&self.field_data).unwrap().1,
            )),
            TextureReferenceFragment::TYPE_ID => Box::new(FragmentType::TextureReference(
                TextureReferenceFragment::parse(&self.field_data).unwrap().1,
            )),
            MeshReferenceFragment::TYPE_ID => Box::new(FragmentType::MeshReference(
                MeshReferenceFragment::parse(&self.field_data).unwrap().1,
            )),
            TextureFragment::TYPE_ID => Box::new(FragmentType::Texture(
                TextureFragment::parse(&self.field_data).unwrap().1,
            )),
            TextureImagesFragment::TYPE_ID => Box::new(FragmentType::TextureImages(
                TextureImagesFragment::parse(&self.field_data).unwrap().1,
            )),
            _ => panic!("Unknown fragment type"),
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.size.to_le_bytes()[..],
            &self.fragment_type.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    use nom::bytes::complete::take;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../fixtures/gfaydark.wld")[..];
        let wld_doc = WldDoc::parse(data).unwrap().1;
        assert_eq!(wld_doc.header.magic, 1414544642);
        assert_eq!(wld_doc.header.version, 0x00015500);
        assert_eq!(wld_doc.header.fragment_count, 4646);
        assert_eq!(wld_doc.header.header_3, 2905);
        assert_eq!(wld_doc.header.header_4, 162660);
        assert_eq!(wld_doc.header.string_hash_size, 52692);
        assert_eq!(wld_doc.header.header_6, 4609);
    }

    //#[test]
    //fn it_serializes() {
    //    let data = &include_bytes!("../../fixtures/gfaydark.wld")[..];
    //    let wld_doc = WldDoc::parse(data).unwrap().1;

    //    let data = &include_bytes!("../../serialized_data.bin")[..];
    //    let (i, header) = WldHeader::parse(&data).unwrap();
    //    //        println!("{:?}", header);

    //    let (i, string_hash_data) =
    //        take::<_, _, nom::error::Error<_>>(header.string_hash_size)(i).unwrap();
    //    //        println!("{:?}", string_hash_data == wld_doc.strings_bytes());
    //    assert_eq!(string_hash_data.len(), wld_doc.strings_bytes().len());

    //    //let (remaining, (string_hash_data, fragment_headers)) = tuple((
    //    //    take(header.string_hash_size),
    //    //    count(FragmentHeader::parse, header.fragment_count as usize),
    //    //))(i)
    //    //.unwrap();
    //    //let wld_doc = WldDoc::parse(data).unwrap().1;

    //    //let serialized_data = wld_doc.into_bytes();
    //    //let mut file = File::create("serialized_data.bin").unwrap();
    //    //file.write_all(&serialized_data).unwrap();
    //    //let result = WldDoc::parse(&serialized_data);
    //    //println!("{:?}", result);
    //    //let deserialized_wld_doc = WldDoc::parse(&serialized_data).unwrap().1;

    //    //assert_eq!(deserialized_wld_doc.header, wld_doc.header);
    //}
}
