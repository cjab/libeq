use std::any::Any;

use super::{Fragment, FragmentParser, StringReference};

use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// Static or animated model reference or player info.
///
/// **Type ID:** 0x14
pub struct ModelFragment {
    pub name_reference: StringReference,

    /// Most flags are _unknown_.
    /// * bit 0 - If set then `unknown_params1` exists.
    /// * bit 1 - If set then `unknown_params2` exists.
    /// * bit 7 - If unset then `unknown_fragment` must contain 0.
    pub flags: u32,

    /// This isn’t really a fragment reference but a string reference.
    /// It points to a “magic” string. When this fragment is used in main zone
    /// files the string is “FLYCAMCALLBACK”. When used as a placeable object reference,
    /// the string is “SPRITECALLBACK”. When creating a 0x14 fragment this is currently
    /// accomplished by creating a fragment reference, setting the fragment to null, and
    /// setting the reference name to the magic string.
    pub name_fragment: StringReference,

    /// Tells how many entries there are.
    pub unknown_params2_count: u32,

    /// Tells how many fragment entries there are.
    pub fragment_count: u32,

    /// _Unknown_
    pub unknown_fragment: u32,

    /// This seems to always contain 0. It seems to only be used in main zone files.
    pub unknown_params1: Option<u32>,

    /// These seem to always contain zeroes. They seem to only be used in main zone files.
    /// There are `unknown_params2_count` of these.
    pub unknown_params2: Option<Vec<u32>>,

    /// Tells how many `unknown_data` pairs there are.
    pub unknown_data_count: u32,

    /// _Unknown_. There are `unknown_data_count` of these.
    pub unknown_data: Vec<(i32, f32)>,

    /// There are `fragment_count` fragment references here. These references can point to several different
    /// kinds of fragments. In main zone files, there seems to be only one entry, which points to
    /// a 0x09 Camera Reference fragment. When this is instead a static object reference, the entry
    /// points to either a 0x2D Mesh Reference fragment. If this is an animated (mob) object
    /// reference, it points to a 0x11 Skeleton Track Set Reference fragment.
    /// This also has been seen to point to a 0x07 Two-dimensional Object Reference fragment
    /// (e.g. coins and blood spots).
    pub fragments: Vec<u32>,

    /// The number of bytes in the name field.
    pub name_size: u32,

    /// An encoded string. It's purpose and possible values are unknown.
    pub name: Vec<u8>,
}

impl FragmentParser for ModelFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x14;
    const TYPE_NAME: &'static str = "Model";

    fn parse(input: &[u8]) -> IResult<&[u8], ModelFragment> {
        let (
            i,
            (
                name_reference,
                flags,
                name_fragment,
                unknown_params2_count,
                fragment_count,
                unknown_fragment,
            ),
        ) = tuple((
            StringReference::parse,
            le_u32,
            StringReference::parse,
            le_u32,
            le_u32,
            le_u32,
        ))(input)?;

        let (i, unknown_params1) = if flags & 0x01 == 0x01 {
            le_u32(i).map(|(i, params1)| (i, Some(params1)))?
        } else {
            (i, None)
        };

        let (i, unknown_params2) = if flags & 0x02 == 0x02 {
            count(le_u32, unknown_params2_count as usize)(i)
                .map(|(i, params2)| (i, Some(params2)))?
        } else {
            (i, None)
        };

        let (i, unknown_data_count) = le_u32(i)?;

        let (i, (unknown_data, fragments, name_size)) = tuple((
            count(tuple((le_i32, le_f32)), unknown_data_count as usize),
            count(le_u32, fragment_count as usize),
            le_u32,
        ))(i)?;

        let (remaining, name) = count(le_u8, name_size as usize)(i)?;

        Ok((
            remaining,
            ModelFragment {
                name_reference,
                flags,
                name_fragment,
                unknown_params2_count,
                fragment_count,
                unknown_fragment,
                unknown_params1,
                unknown_params2,
                unknown_data_count,
                unknown_data,
                fragments,
                name_size,
                name,
            },
        ))
    }
}

impl Fragment for ModelFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.name_fragment.serialize()[..],
            &self.unknown_params2_count.to_le_bytes()[..],
            &self.fragment_count.to_le_bytes()[..],
            &self.unknown_fragment.to_le_bytes()[..],
            &self
                .unknown_params1
                .map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self
                .unknown_params2
                .as_ref()
                .map_or(vec![], |p| p.iter().flat_map(|x| x.to_le_bytes()).collect())[..],
            &self.unknown_data_count.to_le_bytes()[..],
            &self
                .unknown_data
                .iter()
                .flat_map(|d| [d.0.to_le_bytes(), d.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .fragments
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &self.name_size.to_le_bytes()[..],
            &self
                .name
                .iter()
                .flat_map(|n| n.to_le_bytes())
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name_ref(&self) -> &StringReference {
        &self.name_reference
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4639-0x14.frag")[..];
        let frag = ModelFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-52594));
        assert_eq!(frag.flags, 0);
        assert_eq!(frag.name_fragment, 4294914717);
        assert_eq!(frag.unknown_params2_count, 1);
        assert_eq!(frag.fragment_count, 1);
        assert_eq!(frag.unknown_fragment, 0);
        assert_eq!(frag.unknown_params1, None);
        assert_eq!(frag.unknown_params2, None);
        assert_eq!(frag.unknown_data_count, 1);
        assert_eq!(frag.unknown_data, vec![(0, 1e30)]);
        assert_eq!(frag.fragments, vec![4639]);
        assert_eq!(frag.name_size, 0);
        assert_eq!(frag.name, vec![]);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4639-0x14.frag")[..];
        let frag = ModelFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
