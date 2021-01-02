mod fragments;
mod strings;

use core::fmt::Debug;

use log::error;
use nom::bytes::complete::take;
use nom::error::ErrorKind;
use nom::multi::count;
use nom::number::complete::{le_i32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

pub use fragments::{Fragment, FragmentRef, MaterialFragment, MeshFragment, TextureFragment};
pub use strings::{decode_string, StringHash};

pub const MESH_FRAGMENT_ID: u32 = 0x36;
pub const MATERIAL_FRAGMENT_ID: u32 = 0x30;

#[derive(Debug)]
pub enum Error {
    Parser,
}

impl From<nom::Err<(&[u8], ErrorKind)>> for Error {
    fn from(_e: nom::Err<(&[u8], ErrorKind)>) -> Self {
        Self::Parser
    }
}

#[derive(Debug)]
pub struct WldDoc<'a> {
    header: WldHeader,
    pub strings: StringHash,
    pub fragments: Vec<FragmentHeader<'a>>,
}

impl<'a> WldDoc<'a> {
    pub fn parse(input: &[u8]) -> IResult<&[u8], WldDoc> {
        let (i, header) = WldHeader::parse(input)?;
        let (remaining, (string_hash_data, fragments)) = tuple((
            take(header.string_hash_size),
            count(FragmentHeader::parse, header.fragment_count as usize),
        ))(i)?;
        let strings = StringHash::new(string_hash_data);

        Ok((
            remaining,
            WldDoc {
                header,
                strings,
                fragments,
            },
        ))
    }

    /// Get a fragment at a specific index in the wld file.
    pub fn at<T: Fragment<T = T> + Debug>(&self, idx: i32) -> Option<(Option<&str>, T)> {
        self.get(&FragmentRef::<T>::new(idx))
    }

    /// Get a fragment given a fragment reference.
    pub fn get<T: Fragment<T = T> + Debug>(
        &self,
        fragment_ref: &FragmentRef<T>,
    ) -> Option<(Option<&str>, T)> {
        if fragment_ref.is_name_ref() {
            self.get_by_name_ref(fragment_ref)
        } else if fragment_ref.is_index_ref() && fragment_ref.0 < self.fragments.len() as i32 {
            self.get_by_index_ref(fragment_ref)
        } else {
            error!("Fragment reference [{:?}] out of bounds!", fragment_ref);
            None
        }
    }

    fn get_by_index_ref<T: Fragment<T = T> + Debug>(
        &self,
        fragment_ref: &FragmentRef<T>,
    ) -> Option<(Option<&str>, T)> {
        let idx = (fragment_ref.0 - 1) as usize;
        let fragment = self
            .fragments
            .get(idx)
            .expect(&format!("Could not find fragment at {}", idx));
        let name = self.strings.get(fragment.name_reference);
        T::parse(fragment.field_data).map(|r| (name, r.1)).ok()
    }

    fn get_by_name_ref<T: Fragment<T = T> + Debug>(
        &self,
        fragment_ref: &FragmentRef<T>,
    ) -> Option<(Option<&str>, T)> {
        let string_ref = -fragment_ref.0 - 1;
        if let Some(target_name) = self.strings.get(string_ref) {
            self.fragments
                .iter()
                .find(|f| f.name(self).map_or(false, |name| name == target_name))
                .and_then(|f| {
                    let name = self.strings.get(f.name_reference);
                    T::parse(f.field_data).map(|r| (name, r.1)).ok()
                })
        } else {
            None
        }
    }

    /// Iterate over all mesh fragments in the wld file.
    pub(super) fn meshes(&self) -> impl Iterator<Item = (Option<&str>, MeshFragment)> + '_ {
        self.fragment_iter::<MeshFragment>(MESH_FRAGMENT_ID)
    }

    /// Iterate over all material fragments in the wld file.
    pub(super) fn materials(&self) -> impl Iterator<Item = (Option<&str>, MaterialFragment)> + '_ {
        self.fragment_iter::<MaterialFragment>(MATERIAL_FRAGMENT_ID)
    }

    fn fragment_iter<T: Fragment<T = T> + Debug>(
        &self,
        fragment_type: u32,
    ) -> impl Iterator<Item = (Option<&str>, T)> + '_ {
        self.fragments
            .iter()
            .enumerate()
            .filter(move |(_, f)| f.fragment_type == fragment_type)
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
}

type FragmentTypeId = u32;

#[derive(Debug)]
/// All fragments begin with the following header
pub struct FragmentHeader<'a> {
    /// The size of the fragment in bytes. All fragments are padded such that `size`
    /// is evenly divisible by 4 and Size should reflect the padded value.
    size: u32,

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
    pub name_reference: i32,

    pub field_data: &'a [u8],
}

impl<'a> FragmentHeader<'a> {
    pub fn parse(input: &'a [u8]) -> IResult<&[u8], FragmentHeader> {
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
                name_reference,
                fragment_type,
                field_data,
            },
        ))
    }

    pub fn name(&self, doc: &'a WldDoc) -> Option<&'a str> {
        doc.strings.get(self.name_reference)
    }
}
