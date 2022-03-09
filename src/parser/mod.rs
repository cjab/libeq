pub mod fragments;
mod strings;

use core::fmt::Debug;

use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_i32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

pub use fragments::*;
pub use strings::{decode_string, StringHash, StringReference};

#[derive(Debug)]
pub enum FragmentType {
    AlternateMesh(Box<AlternateMeshFragment>),
    VertexColorReference(Box<VertexColorReferenceFragment>),
    VertexColor(Box<VertexColorFragment>),
    MeshAnimatedVertices(Box<MeshAnimatedVerticesFragment>),
    MeshAnimatedVerticesReference(Box<MeshAnimatedVerticesReferenceFragment>),
    AmbientLight(Box<AmbientLightFragment>),
    RegionFlag(Box<RegionFlagFragment>),
    LightInfo(Box<LightInfoFragment>),
    LightSourceReference(Box<LightSourceReferenceFragment>),
    LightSource(Box<LightSourceFragment>),
    PolygonAnimationReference(Box<PolygonAnimationReferenceFragment>),
    PolygonAnimation(Box<PolygonAnimationFragment>),
    First(Box<FirstFragment>),
    ZoneUnknown(Box<ZoneUnknownFragment>),
    SkeletonTrackSetReference(Box<SkeletonTrackSetReferenceFragment>),
    CameraReference(Box<CameraReferenceFragment>),
    Camera(Box<CameraFragment>),
    TwoDimensionalObjectReference(Box<TwoDimensionalObjectReferenceFragment>),
    TwoDimensionalObject(Box<TwoDimensionalObjectFragment>),
    ObjectLocation(Box<ObjectLocationFragment>),
    MobSkeletonPieceTrackReference(Box<MobSkeletonPieceTrackReferenceFragment>),
    MobSkeletonPieceTrack(Box<MobSkeletonPieceTrackFragment>),
    SkeletonTrackSet(Box<SkeletonTrackSetFragment>),
    Model(Box<ModelFragment>),
    BspTree(Box<BspTreeFragment>),
    BspRegion(Box<BspRegionFragment>),
    Mesh(Box<MeshFragment>),
    MaterialList(Box<MaterialListFragment>),
    Material(Box<MaterialFragment>),
    TextureReference(Box<TextureReferenceFragment>),
    MeshReference(Box<MeshReferenceFragment>),
    Texture(Box<TextureFragment>),
    TextureImages(Box<TextureImagesFragment>),
    Unknown(u32),
}

impl FragmentType {
    // TODO: Fix error handling here
    fn from_header(header: &FragmentHeader) -> FragmentType {
        match header.fragment_type {
            AlternateMeshFragment::TYPE_ID => FragmentType::AlternateMesh(Box::new(
                AlternateMeshFragment::parse(&header.field_data).unwrap().1,
            )),
            VertexColorReferenceFragment::TYPE_ID => FragmentType::VertexColorReference(Box::new(
                VertexColorReferenceFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            VertexColorFragment::TYPE_ID => FragmentType::VertexColor(Box::new(
                VertexColorFragment::parse(&header.field_data).unwrap().1,
            )),
            MeshAnimatedVerticesFragment::TYPE_ID => FragmentType::MeshAnimatedVertices(Box::new(
                MeshAnimatedVerticesFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            MeshAnimatedVerticesReferenceFragment::TYPE_ID => {
                FragmentType::MeshAnimatedVerticesReference(Box::new(
                    MeshAnimatedVerticesReferenceFragment::parse(&header.field_data)
                        .unwrap()
                        .1,
                ))
            }
            AmbientLightFragment::TYPE_ID => FragmentType::AmbientLight(Box::new(
                AmbientLightFragment::parse(&header.field_data).unwrap().1,
            )),
            RegionFlagFragment::TYPE_ID => FragmentType::RegionFlag(Box::new(
                RegionFlagFragment::parse(&header.field_data).unwrap().1,
            )),
            LightInfoFragment::TYPE_ID => FragmentType::LightInfo(Box::new(
                LightInfoFragment::parse(&header.field_data).unwrap().1,
            )),
            LightSourceReferenceFragment::TYPE_ID => FragmentType::LightSourceReference(Box::new(
                LightSourceReferenceFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            LightSourceFragment::TYPE_ID => FragmentType::LightSource(Box::new(
                LightSourceFragment::parse(&header.field_data).unwrap().1,
            )),
            PolygonAnimationReferenceFragment::TYPE_ID => {
                FragmentType::PolygonAnimationReference(Box::new(
                    PolygonAnimationReferenceFragment::parse(&header.field_data)
                        .unwrap()
                        .1,
                ))
            }
            PolygonAnimationFragment::TYPE_ID => FragmentType::PolygonAnimation(Box::new(
                PolygonAnimationFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            FirstFragment::TYPE_ID => FragmentType::First(Box::new(
                FirstFragment::parse(&header.field_data).unwrap().1,
            )),
            ZoneUnknownFragment::TYPE_ID => FragmentType::ZoneUnknown(Box::new(
                ZoneUnknownFragment::parse(&header.field_data).unwrap().1,
            )),
            SkeletonTrackSetReferenceFragment::TYPE_ID => {
                FragmentType::SkeletonTrackSetReference(Box::new(
                    SkeletonTrackSetReferenceFragment::parse(&header.field_data)
                        .unwrap()
                        .1,
                ))
            }
            CameraReferenceFragment::TYPE_ID => FragmentType::CameraReference(Box::new(
                CameraReferenceFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            CameraFragment::TYPE_ID => FragmentType::Camera(Box::new(
                CameraFragment::parse(&header.field_data).unwrap().1,
            )),
            TwoDimensionalObjectReferenceFragment::TYPE_ID => {
                FragmentType::TwoDimensionalObjectReference(Box::new(
                    TwoDimensionalObjectReferenceFragment::parse(&header.field_data)
                        .unwrap()
                        .1,
                ))
            }
            TwoDimensionalObjectFragment::TYPE_ID => FragmentType::TwoDimensionalObject(Box::new(
                TwoDimensionalObjectFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            ObjectLocationFragment::TYPE_ID => FragmentType::ObjectLocation(Box::new(
                ObjectLocationFragment::parse(&header.field_data).unwrap().1,
            )),
            MobSkeletonPieceTrackReferenceFragment::TYPE_ID => {
                FragmentType::MobSkeletonPieceTrackReference(Box::new(
                    MobSkeletonPieceTrackReferenceFragment::parse(&header.field_data)
                        .unwrap()
                        .1,
                ))
            }
            MobSkeletonPieceTrackFragment::TYPE_ID => {
                FragmentType::MobSkeletonPieceTrack(Box::new(
                    MobSkeletonPieceTrackFragment::parse(&header.field_data)
                        .unwrap()
                        .1,
                ))
            }
            SkeletonTrackSetFragment::TYPE_ID => FragmentType::SkeletonTrackSet(Box::new(
                SkeletonTrackSetFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            ModelFragment::TYPE_ID => FragmentType::Model(Box::new(
                ModelFragment::parse(&header.field_data).unwrap().1,
            )),
            BspTreeFragment::TYPE_ID => FragmentType::BspTree(Box::new(
                BspTreeFragment::parse(&header.field_data).unwrap().1,
            )),
            BspRegionFragment::TYPE_ID => FragmentType::BspRegion(Box::new(
                BspRegionFragment::parse(&header.field_data).unwrap().1,
            )),
            MeshFragment::TYPE_ID => {
                FragmentType::Mesh(Box::new(MeshFragment::parse(&header.field_data).unwrap().1))
            }
            MaterialListFragment::TYPE_ID => FragmentType::MaterialList(Box::new(
                MaterialListFragment::parse(&header.field_data).unwrap().1,
            )),
            MaterialFragment::TYPE_ID => FragmentType::Material(Box::new(
                MaterialFragment::parse(&header.field_data).unwrap().1,
            )),
            TextureReferenceFragment::TYPE_ID => FragmentType::TextureReference(Box::new(
                TextureReferenceFragment::parse(&header.field_data)
                    .unwrap()
                    .1,
            )),
            MeshReferenceFragment::TYPE_ID => FragmentType::MeshReference(Box::new(
                MeshReferenceFragment::parse(&header.field_data).unwrap().1,
            )),
            TextureFragment::TYPE_ID => FragmentType::Texture(Box::new(
                TextureFragment::parse(&header.field_data).unwrap().1,
            )),
            TextureImagesFragment::TYPE_ID => FragmentType::TextureImages(Box::new(
                TextureImagesFragment::parse(&header.field_data).unwrap().1,
            )),
            type_id => FragmentType::Unknown(type_id),
        }
    }
}

#[derive(Debug)]
pub struct WldDoc<'a> {
    header: WldHeader,
    pub strings: StringHash,
    pub fragment_headers: Vec<FragmentHeader<'a>>,
    pub fragments: Vec<FragmentType>,
}

impl<'a> WldDoc<'a> {
    pub fn parse(input: &[u8]) -> IResult<&[u8], WldDoc> {
        let (i, header) = WldHeader::parse(input)?;
        let (remaining, (string_hash_data, fragment_headers)) = tuple((
            take(header.string_hash_size),
            count(FragmentHeader::parse, header.fragment_count as usize),
        ))(i)?;
        let strings = StringHash::new(string_hash_data);

        let fragments: Vec<FragmentType> = fragment_headers
            .iter()
            .map(FragmentType::from_header)
            .collect();

        Ok((
            remaining,
            WldDoc {
                header,
                strings,
                fragment_headers,
                fragments,
            },
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        [
            self.header.serialize(),
            self.strings.serialize(),
            self.fragment_headers
                .iter()
                .flat_map(|f| f.serialize())
                .collect(),
        ]
        .concat()
    }

    /// Get a fragment given a fragment reference.
    pub fn get<T: Fragment<T = T> + Debug>(
        &self,
        fragment_ref: &FragmentRef<T>,
    ) -> Option<(Option<&str>, T)> {
        match fragment_ref {
            FragmentRef::Name(_, _) => self.get_by_name_ref(fragment_ref),
            FragmentRef::Index(_, _) => self.get_by_index_ref(fragment_ref),
        }
    }

    fn get_by_index_ref<T: Fragment<T = T> + Debug>(
        &self,
        fragment_ref: &FragmentRef<T>,
    ) -> Option<(Option<&str>, T)> {
        let idx = if let FragmentRef::Index(idx, _) = fragment_ref {
            idx
        } else {
            return None;
        };

        let fragment = self.fragment_headers.get((idx - 1) as usize)?;
        let name = fragment.name_reference.and_then(|r| self.strings.get(r));
        T::parse(&fragment.field_data).map(|r| (name, r.1)).ok()
    }

    fn get_by_name_ref<T: Fragment<T = T> + Debug>(
        &self,
        fragment_ref: &FragmentRef<T>,
    ) -> Option<(Option<&str>, T)> {
        let name_ref = if let FragmentRef::Name(name_ref, _) = fragment_ref {
            *name_ref
        } else {
            return None;
        };

        if let Some(target_name) = self.strings.get(name_ref) {
            self.fragment_headers
                .iter()
                .find(|f| f.name(self).map_or(false, |name| name == target_name))
                .and_then(|f| {
                    let name = f.name_reference.and_then(|r| self.strings.get(r));
                    T::parse(&f.field_data).map(|r| (name, r.1)).ok()
                })
        } else {
            None
        }
    }

    /// Iterate over all mesh fragments in the wld file.
    pub(super) fn meshes(&self) -> impl Iterator<Item = (Option<&str>, MeshFragment)> + '_ {
        self.fragment_iter::<MeshFragment>()
    }

    /// Iterate over all material fragments in the wld file.
    pub(super) fn materials(&self) -> impl Iterator<Item = (Option<&str>, MaterialFragment)> + '_ {
        self.fragment_iter::<MaterialFragment>()
    }

    pub fn fragment_iter<T: Fragment<T = T> + Debug>(
        &self,
    ) -> impl Iterator<Item = (Option<&str>, T)> + '_ {
        self.fragment_headers
            .iter()
            .enumerate()
            .filter(move |(_, f)| f.fragment_type == T::TYPE_ID)
            .map(|(i, _)| FragmentRef::new((i + 1) as i32))
            .filter_map(move |r| self.get(&r))
    }
}

/// This header is present at the beginning of every .wld file.
#[derive(Debug)]
struct WldHeader {
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
    fn parse(input: &[u8]) -> IResult<&[u8], WldHeader> {
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

    pub fn serialize(&self) -> Vec<u8> {
        [
            self.magic,
            self.version,
            self.fragment_count,
            self.header_3,
            self.header_4,
            self.string_hash_size,
            self.header_6,
        ]
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
    }
}

type FragmentTypeId = u32;

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
    pub name_reference: Option<StringReference>,

    pub field_data: &'a [u8],
}

impl<'a> FragmentHeader<'a> {
    pub fn parse(input: &[u8]) -> IResult<&[u8], FragmentHeader> {
        let (i, (size, fragment_type, name_reference)) = tuple((le_u32, le_u32, le_i32))(input)?;

        let (remaining, field_data) = if fragment_type != 0x35 {
            take(size - 4)(i)? // TODO: What are the extra 4 bytes for?
        } else {
            (i, &[] as &[u8])
        };

        Ok((
            remaining,
            FragmentHeader {
                size,
                name_reference: StringReference::new(name_reference),
                fragment_type,
                field_data,
            },
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        let result = [
            self.size.to_le_bytes(),
            self.fragment_type.to_le_bytes(),
            self.name_reference
                .map_or(0, |r| r.serialize())
                .to_le_bytes(),
        ]
        .concat();
        result
    }

    pub fn name(&self, doc: &'a WldDoc) -> Option<&'a str> {
        self.name_reference.and_then(|r| doc.strings.get(r))
    }
}
