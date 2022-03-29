mod alternate_mesh;
mod mesh_animated_vertices;
mod vertex_color;
mod vertex_color_reference;

use std::any::Any;
use std::marker::PhantomData;

use nom::combinator::map;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_i32, le_i8, le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::{decode_string, StringReference};

pub use alternate_mesh::*;
pub use mesh_animated_vertices::*;
pub use vertex_color::*;
pub use vertex_color_reference::*;

#[derive(Debug, Clone, Copy)]
pub enum FragmentRef<T> {
    Name(StringReference, PhantomData<T>),
    Index(u32, PhantomData<T>),
}

impl<T> FragmentRef<T> {
    pub fn new(idx: i32) -> FragmentRef<T> {
        match StringReference::new(idx) {
            Some(name_ref) => FragmentRef::Name(name_ref, PhantomData),
            None => FragmentRef::Index(idx as u32, PhantomData),
        }
    }

    pub fn serialize(&self) -> i32 {
        match self {
            Self::Name(string_ref, _) => string_ref.serialize(),
            Self::Index(idx, _) => idx as i32,
        }
    }
}

pub trait Fragment {
    fn serialize(&self) -> Vec<u8>;
    fn as_any(&self) -> &dyn Any;
}

pub trait FragmentType {
    type T;
    const TYPE_ID: u32;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::T>;
}

fn fragment_ref<T>(input: &[u8]) -> IResult<&[u8], FragmentRef<T>> {
    let (remaining, frag_ref_idx) = le_i32(input)?;
    Ok((remaining, FragmentRef::new(frag_ref_idx)))
}

#[derive(Debug)]
/// A reference to a [MeshAnimatedVerticesFragment].
///
/// **Type ID:** 0x2f
pub struct MeshAnimatedVerticesReferenceFragment {
    /// The [MeshAnimatedVerticesFragment] reference.
    pub reference: FragmentRef<MeshAnimatedVerticesFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentType for MeshAnimatedVerticesReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2f;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshAnimatedVerticesReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            MeshAnimatedVerticesReferenceFragment { reference, flags },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [LightSourceReferenceFragment].
///
/// **Type ID:** 0x2a
pub struct AmbientLightFragment {
    /// The [LightSourceReferenceFragment] reference.
    pub reference: FragmentRef<LightSourceReferenceFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,

    /// The number of region ids.
    pub region_count: u32,

    /// There are `region_count` region ids here. Each isn’t a fragment reference
    /// per se, but the ID of a 0x22 BSP region fragment. For example, if there are
    /// 100 0x22 BSP Region fragments, then the possible values are in the range 0-99.
    /// This constitutes a list of regions that have the ambient lighting given by the
    /// 0x1C fragment that this fragment references.
    pub regions: Vec<u32>,
}

impl FragmentType for AmbientLightFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2a;

    fn parse(input: &[u8]) -> IResult<&[u8], AmbientLightFragment> {
        let (i, (reference, flags, region_count)) = tuple((fragment_ref, le_u32, le_u32))(input)?;
        let (remaining, regions) = count(le_u32, region_count as usize)(i)?;

        Ok((
            remaining,
            AmbientLightFragment {
                reference,
                flags,
                region_count,
                regions,
            },
        ))
    }
}

#[derive(Debug)]
/// This fragment lets you flag certain regions (as defined by 0x22 BSP Region fragments)
/// in a particular way. The flagging is done by setting the name of this fragment to a
/// particular “magic” value.
///
/// The possible values are:
///
/// * WT_ZONE ...............Flag all regions in the list as underwater regions.
/// * LA_ZONE ...............Flag all regions in the list as lava regions.
/// * DRP_ZONE ..............Flag all regions in the list as PvP regions.
/// * DRNTP##########_ZONE...Flag all regions in the list as zone point regions.
///                          The ####’s are actually numbers and hyphens that somehow tell
///                          the client the zone destination. This method of setting zone
///                          points may or may not be obsolete.
///
/// **Type ID:** 0x29
pub struct RegionFlagFragment {
    /// _Unknown_ - Usually contains 0.
    pub flags: u32,

    /// The number of region ids.
    pub region_count: u32,

    /// There are `region_count` regions. Each isn’t a fragment reference per se, but the
    /// ID of a 0x22 BSP region fragment. For example, if there are 100 0x22 BSP Region
    /// fragments, then the possible values are in the range 0-99. This constitutes a
    /// list of regions that are to be flagged in the particular way.
    pub regions: Vec<u32>,

    /// The number of bytes following in the `data2` field.
    pub size2: u32,

    /// An encoded string. An alternate way of using this fragment is to call this fragment
    /// Z####_ZONE, where #### is a four- digit number starting with zero. Then Data2 would
    /// contain a “magic” string that told the client what was special about the included
    /// regions (e.g. WTN__01521000000000000000000000___000000000000). This field is padded
    /// with nulls to make it end on a DWORD boundary.
    pub data2: Vec<u8>,
}

impl FragmentType for RegionFlagFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x29;

    fn parse(input: &[u8]) -> IResult<&[u8], RegionFlagFragment> {
        let (i, (flags, region_count)) = tuple((le_u32, le_u32))(input)?;
        let (i, (regions, size2)) = tuple((count(le_u32, region_count as usize), le_u32))(i)?;

        let padding = (4 - size2 % 4) % 4;
        let size2_with_padding = size2 + padding;
        let (remaining, data2) = count(le_u8, size2_with_padding as usize)(i)?;

        Ok((
            remaining,
            RegionFlagFragment {
                flags,
                region_count,
                regions,
                size2,
                data2,
            },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [LightSourceReferenceFragment].
///
/// **Type ID:** 0x28
pub struct LightInfoFragment {
    /// The [LightSourceReferenceFragment] reference.
    pub reference: FragmentRef<LightSourceReferenceFragment>,

    /// _Unknown_ - Usually contains 256 (0x100).
    pub flags: u32,

    /// X component of the light location.
    pub x: f32,

    /// Y component of the light location.
    pub y: f32,

    /// Z component of the light location.
    pub z: f32,

    /// Contains the light radius.
    pub radius: f32,
}

impl FragmentType for LightInfoFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x28;

    fn parse(input: &[u8]) -> IResult<&[u8], LightInfoFragment> {
        let (remaining, (reference, flags, x, y, z, radius)) =
            tuple((fragment_ref, le_u32, le_f32, le_f32, le_f32, le_f32))(input)?;
        Ok((
            remaining,
            LightInfoFragment {
                reference,
                flags,
                x,
                y,
                z,
                radius,
            },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [LightSourceFragment].
///
/// **Type ID:** 0x1c
pub struct LightSourceReferenceFragment {
    /// The [LightSourceFragment] reference.
    pub reference: FragmentRef<LightSourceFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentType for LightSourceReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x1c;

    fn parse(input: &[u8]) -> IResult<&[u8], LightSourceReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((remaining, LightSourceReferenceFragment { reference, flags }))
    }
}

#[derive(Debug)]
/// **Type ID:** 0x1b
pub struct LightSourceFragment {
    /// _Unknown_
    /// * bit 1 - Usually 1 when dealing with placed light sources.
    /// * bit 2 - Usually 1.
    /// * bit 3 - Usually 1 when dealing with placed light source.
    ///           If bit 4 is set then `params3b` only exists if
    ///           this bit is also set (not sure about this).
    /// * bit 4 - If unset `params3a` exists but `params3b` and `red`, `green` and `blue` don't exist.
    ///           Otherwise, `params3a` doesn't exist but `params3b` and `red`, `green` and `blue` do exist.
    ///           This flag seems to determine whether the light is just a simple white light
    ///           or a light with its own color values.
    pub flags: u32,

    /// _Unknown_ - Usually contains 1
    pub params2: u32,

    /// _Unknown_ - Usually contains 1
    pub params3a: Option<f32>,

    /// _Unknown_ - Usually contains 200 (attenuation?).
    pub params3b: Option<u32>,

    /// _Unknown_ - Usually contains 1.
    pub params4: Option<u8>,

    /// Red component, scaled from 0 (no red component) to 1 (100% red).
    pub red: Option<u8>,

    /// Green component, scaled from 0 (no green component) to 1 (100% green).
    pub green: Option<u8>,

    /// Blue component, scaled from 0 (no blue component) to 1 (100% blue).
    pub blue: Option<u8>,
}

impl FragmentType for LightSourceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x1b;

    fn parse(input: &[u8]) -> IResult<&[u8], LightSourceFragment> {
        let (i, (flags, params2)) = tuple((le_u32, le_u32))(input)?;

        let (i, params3a) = if flags & 0x10 == 0x00 {
            le_f32(i).map(|(i, params3a)| (i, Some(params3a)))?
        } else {
            (i, None)
        };

        let (i, params3b) = if flags & 0x18 == 0x18 {
            le_u32(i).map(|(i, params3b)| (i, Some(params3b)))?
        } else {
            (i, None)
        };

        let (remaining, (params4, red, green, blue)) = if flags & 0x10 == 0x10 {
            tuple((le_u8, le_u8, le_u8, le_u8))(i)
                .map(|(i, (p4, r, g, b))| (i, (Some(p4), Some(r), Some(g), Some(b))))?
        } else {
            (i, (None, None, None, None))
        };

        Ok((
            remaining,
            LightSourceFragment {
                flags,
                params2,
                params3a,
                params3b,
                params4,
                red,
                green,
                blue,
            },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [PolygonAnimationFragment].
///
/// **Type ID:** 0x18
pub struct PolygonAnimationReferenceFragment {
    /// The [PolygonAnimationFragment] reference.
    pub reference: FragmentRef<PolygonAnimationFragment>,

    /// _Unknown_
    /// * bit 0 - If set `params1` exists.
    pub flags: u32,

    /// _Unknown_
    pub params1: f32,
}

impl FragmentType for PolygonAnimationReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x18;

    fn parse(input: &[u8]) -> IResult<&[u8], PolygonAnimationReferenceFragment> {
        let (remaining, (reference, flags, params1)) =
            tuple((fragment_ref, le_u32, le_f32))(input)?;
        Ok((
            remaining,
            PolygonAnimationReferenceFragment {
                reference,
                flags,
                params1,
            },
        ))
    }
}

#[derive(Debug)]
/// **Type ID:** 0x17
pub struct PolygonAnimationFragment {
    /// _Unknown_ - Usually contains 0.1
    pub params1: f32,

    /// _Unknown_
    /// * bit 0 - If unset `params2` must be 1.0
    pub flags: u32,

    /// The number of `entries1` entries.
    pub size1: u32,

    /// The number of `entries2` entries.
    pub size2: u32,

    /// _Unknown_
    pub params2: f32,

    /// _Unknown_ - Usually contains 1.0
    pub params3: f32,

    /// _Unknown_ - There are size1 of these.
    pub entries1: Vec<(f32, f32, f32)>,

    /// _Unknown_ - There are size2 of these.
    ///
    /// Tuple is as follows:
    /// (number of entries in data, data vec)
    ///
    /// The data appears to be indices into the X, Y, Z entries above.
    pub entries2: Vec<(u32, Vec<u32>)>,
}

impl FragmentType for PolygonAnimationFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x17;

    fn parse(input: &[u8]) -> IResult<&[u8], PolygonAnimationFragment> {
        let (i, (params1, flags, size1, size2, params2, params3)) =
            tuple((le_f32, le_u32, le_u32, le_u32, le_f32, le_f32))(input)?;

        let entry2 = |input| {
            let (i, entry_size) = le_u32(input)?;
            let (i, entries) = count(le_u32, entry_size as usize)(i)?;
            Ok((i, (entry_size, entries)))
        };

        let (remaining, (entries1, entries2)) = tuple((
            count(tuple((le_f32, le_f32, le_f32)), size1 as usize),
            count(entry2, size2 as usize),
        ))(i)?;

        Ok((
            remaining,
            PolygonAnimationFragment {
                params1,
                flags,
                size1,
                size2,
                params2,
                params3,
                entries1,
                entries2,
            },
        ))
    }
}

#[derive(Debug)]
/// There are no fields.
///
/// **Type ID:** 0x35
pub struct FirstFragment {}

impl FragmentType for FirstFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x35;

    fn parse(input: &[u8]) -> IResult<&[u8], FirstFragment> {
        Ok((input, FirstFragment {}))
    }
}

#[derive(Debug)]
/// _Unknown_
///
/// **Type ID:** 0x16
pub struct ZoneUnknownFragment {}

impl FragmentType for ZoneUnknownFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x16;

    fn parse(input: &[u8]) -> IResult<&[u8], ZoneUnknownFragment> {
        Ok((input, ZoneUnknownFragment {}))
    }
}

#[derive(Debug)]
/// A reference to a [SkeletonTrackSetFragment].
///
/// **Type ID:** 0x11
pub struct SkeletonTrackSetReferenceFragment {
    /// The [SkeletonTrackSetFragment] reference.
    pub reference: FragmentRef<SkeletonTrackSetFragment>,

    /// _Unknown_ Seems to always contain 0.
    pub params1: u32,
}

impl FragmentType for SkeletonTrackSetReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x11;

    fn parse(input: &[u8]) -> IResult<&[u8], SkeletonTrackSetReferenceFragment> {
        let (remaining, (reference, params1)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            SkeletonTrackSetReferenceFragment { reference, params1 },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [CameraFragment].
///
/// **Type ID:** 0x09
pub struct CameraReferenceFragment {
    /// The [CameraFragment] reference.
    pub reference: FragmentRef<CameraFragment>,

    /// _Unknown_ Seems to always contain 0.
    pub flags: u32,
}

impl FragmentType for CameraReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x09;

    fn parse(input: &[u8]) -> IResult<&[u8], CameraReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((remaining, CameraReferenceFragment { reference, flags }))
    }
}

#[derive(Debug)]
/// This fragment is poorly understood. It seems to contain 26 parameters, some of which
/// are DWORDS (32-bit integers) and some of which are FLOATS (32-bit floating-point values).
/// Until more is known, they are here described as Params[0..25] and their known values
/// are documented.
///
/// In main zone files, the name of this fragment always seems to be CAMERA_DUMMY.
///
/// **Type ID:** 0x08
pub struct CameraFragment {
    /// _Unknown_ - Usually 0
    pub params0: u32,

    /// _Unknown_ - Usually 0
    pub params1: u32,

    /// _Unknown_ - Usually 1
    pub params2: f32,

    /// _Unknown_ - Usually 0
    pub params3: u32,

    /// _Unknown_ - Usually 0
    pub params4: u32,

    /// _Unknown_ - Usually -1.0
    pub params5: f32,

    /// _Unknown_ - Usually 1.0
    pub params6: f32,

    /// _Unknown_ - Usually 0
    pub params7: u32,

    /// _Unknown_ - Usually 1.0
    pub params8: f32,

    /// _Unknown_ - Usually 1.0
    pub params9: f32,

    /// _Unknown_ - Usually 0
    pub params10: u32,

    /// _Unknown_ - Usually 1.0
    pub params11: f32,

    /// _Unknown_ - Usually -1.0
    pub params12: f32,

    /// _Unknown_ - Usually 0
    pub params13: u32,

    /// _Unknown_ - Usually -1.0
    pub params14: f32,

    /// _Unknown_ - Usually -1.0
    pub params15: f32,

    /// _Unknown_ - Usually 4
    pub params16: u32,

    /// _Unknown_ - Usually 0
    pub params17: u32,

    /// _Unknown_ - Usually 0
    pub params18: u32,

    /// _Unknown_ - Usually 0
    pub params19: u32,

    /// _Unknown_ - Usually 1
    pub params20: u32,

    /// _Unknown_ - Usually 2
    pub params21: u32,

    /// _Unknown_ - Usually 3
    pub params22: u32,

    /// _Unknown_ - Usually 0
    pub params23: u32,

    /// _Unknown_ - Usually 1
    pub params24: u32,

    /// _Unknown_ - Usually 11
    pub params25: u32,
}

impl FragmentType for CameraFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x08;

    fn parse(input: &[u8]) -> IResult<&[u8], CameraFragment> {
        let (
            i,
            (
                params0,
                params1,
                params2,
                params3,
                params4,
                params5,
                params6,
                params7,
                params8,
                params9,
            ),
        ) = tuple((
            le_u32, le_u32, le_f32, le_u32, le_u32, le_f32, le_f32, le_u32, le_f32, le_f32,
        ))(input)?;

        let (
            remaining,
            (
                params10,
                params11,
                params12,
                params13,
                params14,
                params15,
                params16,
                params17,
                params18,
                params19,
                params20,
                params21,
                params22,
                params23,
                params24,
                params25,
            ),
        ) = tuple((
            le_u32, le_f32, le_f32, le_u32, le_f32, le_f32, le_u32, le_u32, le_u32, le_u32, le_u32,
            le_u32, le_u32, le_u32, le_u32, le_u32,
        ))(i)?;

        Ok((
            remaining,
            CameraFragment {
                params0,
                params1,
                params2,
                params3,
                params4,
                params5,
                params6,
                params7,
                params8,
                params9,
                params10,
                params11,
                params12,
                params13,
                params14,
                params15,
                params16,
                params17,
                params18,
                params19,
                params20,
                params21,
                params22,
                params23,
                params24,
                params25,
            },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [TwoDimensionalObjectFragment].
///
/// **Type ID:** 0x07
pub struct TwoDimensionalObjectReferenceFragment {
    /// The [TwoDimensionalObjectFragment] reference.
    pub reference: FragmentRef<TwoDimensionalObjectReferenceFragment>,

    /// _Unknown_ Seems to always contain 0.
    pub flags: u32,
}

impl FragmentType for TwoDimensionalObjectReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x07;

    fn parse(input: &[u8]) -> IResult<&[u8], TwoDimensionalObjectReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            TwoDimensionalObjectReferenceFragment { reference, flags },
        ))
    }
}

#[derive(Debug)]
/// This fragment is rarely used. It describes objects that are purely two-dimensional
/// in nature. Examples are coins and blood spatters.
///
/// **Type ID:** 0x06
pub struct TwoDimensionalObjectFragment {
    pub flags: SpriteFlags,

    /// Windcatcher:
    /// _Unknown_
    /// NEW:
    /// The number of frames that are present in each HEADING block.
    pub num_frames: u32,

    /// Windcatcher:
    /// _Unknown_
    ///
    /// NEW:
    /// The number of PITCH blocks
    pub num_pitches: u32,

    /// Windcatcher:
    /// _Unknown_ - though I suspect it might be the object’s size.
    /// NEW:
    /// The SPRITESIZE %f %f statement
    pub sprite_size: (f32, f32),

    /// Windcatcher:
    /// _Unknown_
    /// NEW:
    /// SPHERE statement, references a 0x22 fragment
    pub sphere_fragment: u32,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 7 of flags is set.
    /// NEW:
    /// DEPTHSCALE statement
    pub depth_scale: Option<f32>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 0 of flags is set.
    /// NEW:
    /// CENTEROFFSET statement
    pub center_offset: Option<(f32, f32, f32)>,

    /// _Unknown_ - Only exists if bit 1 of flags is set.
    pub bounding_radius: Option<f32>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 2 of flags is set.
    /// NEW:
    /// CURRENTFRAME statement
    pub current_frame: Option<u32>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 3 of flags is set.
    /// Typically contains 100.
    ///
    /// NEW:
    /// SLEEP statement
    pub sleep: Option<u32>,

    /// PITCH blocks
    pub pitches: Vec<SpritePitch>,

    /// Windcatcher:
    /// _Unknown_
    /// NEW:
    /// Corresponds to RENDER_METHOD statement.
    pub render_method: RenderMethod,

    /// Mostly _Unknown_
    /// * bit 0 - If set `pen` exists.
    /// * bit 1 - If set `brightness` exists.
    /// * bit 2 - If set `scaled_ambient` exists.
    /// * bit 3 - If set `params7_fragment` exists.
    /// * bit 4 - If set `params7_matrix` exists.
    /// * bit 5 - If set `params7_size` and `params7_data` exist.
    /// * bit 6 - TWOSIDED
    pub render_flags: RenderFlags,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 0 of `renderinfo_flags` is set.
    /// NEW:
    /// Corresponds to PEN in RENDERINFO.
    pub pen: Option<u32>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 1 of `renderinfo_flags` is set.
    /// NEW:
    /// Corresponds to BRIGHTNESS in RENDERINFO.
    pub brightness: Option<f32>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 2 of `renderinfo_flags` is set.
    /// NEW:
    /// Corresponds to SCALEDAMBIENT in RENDERINFO.
    pub scaled_ambient: Option<f32>,

    /// _Unknown_ - Only exists if bit 3 of `renderinfo_flags` is set.
    pub params7_fragment: Option<f32>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 4 of `renderinfo_flags` is set.
    /// It looks like some sort of transformation matrix.
    /// NEW:
    /// Corresponds to UVORIGIN, UAXIS, and VAXIS in RENDERINFO
    pub uv_info: Option<UvInfo>,

    /// _Unknown_ - Only exists if bit 5 of `renderinfo_flags` is set.
    pub params7_size: Option<u32>,

    /// _Unknown_ - Only exists if bit 5 of `renderinfo_flags` is set.
    /// `params7_size` * 2 entries.
    pub params7_data: Option<Vec<u32>>,
}

impl FragmentType for TwoDimensionalObjectFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x06;

    fn parse(input: &[u8]) -> IResult<&[u8], TwoDimensionalObjectFragment> {
        let (i, (flags, num_frames, num_pitches, sprite_size, sphere_fragment)) = tuple((
            SpriteFlags::parse,
            le_u32,
            le_u32,
            tuple((le_f32, le_f32)),
            le_u32,
        ))(input)?;

        let (i, depth_scale) = if flags.has_depth_scale() {
            le_f32(i).map(|(i, p2)| (i, Some(p2)))?
        } else {
            (i, None)
        };

        let (i, center_offset) = if flags.has_center_offset() {
            tuple((le_f32, le_f32, le_f32))(i).map(|(i, p3)| (i, Some(p3)))?
        } else {
            (i, None)
        };

        let (i, bounding_radius) = if flags.has_bounding_radius() {
            le_f32(i).map(|(i, p4)| (i, Some(p4)))?
        } else {
            (i, None)
        };

        let (i, current_frame) = if flags.has_current_frame() {
            le_u32(i).map(|(i, p5)| (i, Some(p5)))?
        } else {
            (i, None)
        };

        let (i, sleep) = if flags.has_sleep() {
            le_u32(i).map(|(i, p6)| (i, Some(p6)))?
        } else {
            (i, None)
        };

        let (i, pitches) = count(
            |input| SpritePitch::parse(num_frames, input),
            num_pitches as usize,
        )(i)?;

        let (i, (render_method, render_flags)) =
            tuple((RenderMethod::parse, RenderFlags::parse))(i)?;

        let (i, pen) = if render_flags.has_pen() {
            le_u32(i).map(|(i, p2)| (i, Some(p2)))?
        } else {
            (i, None)
        };

        let (i, brightness) = if render_flags.has_brightness() {
            le_f32(i).map(|(i, p3)| (i, Some(p3)))?
        } else {
            (i, None)
        };

        let (i, scaled_ambient) = if render_flags.has_scaled_ambient() {
            le_f32(i).map(|(i, p4)| (i, Some(p4)))?
        } else {
            (i, None)
        };

        let (i, params7_fragment) = if render_flags.has_simple_sprite() {
            le_f32(i).map(|(i, f)| (i, Some(f)))?
        } else {
            (i, None)
        };

        let (i, uv_info) = if render_flags.has_uv_info() {
            UvInfo::parse(i).map(|(i, m)| (i, Some(m)))?
        } else {
            (i, None)
        };

        // TODO: This seems wrong.
        let (i, params7_size) = if render_flags.0 & 0x20 == 0x20 {
            le_u32(i).map(|(i, s)| (i, Some(s)))?
        } else {
            (i, None)
        };

        // TODO: This seems wrong.
        let (remaining, params7_data) = match params7_size {
            Some(size) => {
                if render_flags.0 & 0x40 == 0x40 && params7_size.is_some() {
                    count(le_u32, (size * 2) as usize)(i).map(|(i, d)| (i, Some(d)))?
                } else {
                    (i, None)
                }
            }
            _ => (i, None),
        };

        Ok((
            remaining,
            TwoDimensionalObjectFragment {
                flags,
                num_frames,
                num_pitches,
                sprite_size,
                sphere_fragment,
                depth_scale,
                center_offset,
                bounding_radius,
                current_frame,
                sleep,
                pitches,
                render_method,
                render_flags,
                pen,
                brightness,
                scaled_ambient,
                params7_fragment,
                uv_info,
                params7_size,
                params7_data,
            },
        ))
    }
}

#[derive(Debug)]
pub struct UvInfo {
    pub uv_origin: (f32, f32, f32),
    pub u_axis: (f32, f32, f32),
    pub v_axis: (f32, f32, f32),
}

impl FragmentType for UvInfo {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], UvInfo> {
        let (remaining, (uv_origin, u_axis, v_axis)) = tuple((
            tuple((le_f32, le_f32, le_f32)),
            tuple((le_f32, le_f32, le_f32)),
            tuple((le_f32, le_f32, le_f32)),
        ))(input)?;

        Ok((
            remaining,
            UvInfo {
                uv_origin,
                u_axis,
                v_axis,
            },
        ))
    }
}

#[derive(Debug)]
pub struct SpriteFlags(u32);

impl SpriteFlags {
    const HAS_CENTER_OFFSET: u32 = 0x01;
    const HAS_BOUNDING_RADIUS: u32 = 0x02;
    const HAS_CURRENT_FRAME: u32 = 0x04;
    const HAS_SLEEP: u32 = 0x08;
    const SKIP_FRAMES: u32 = 0x40;
    const HAS_DEPTH_SCALE: u32 = 0x80;

    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    pub fn has_center_offset(&self) -> bool {
        self.0 & Self::HAS_CENTER_OFFSET == Self::HAS_CENTER_OFFSET
    }

    pub fn has_bounding_radius(&self) -> bool {
        self.0 & Self::HAS_BOUNDING_RADIUS == Self::HAS_BOUNDING_RADIUS
    }

    pub fn has_current_frame(&self) -> bool {
        self.0 & Self::HAS_CURRENT_FRAME == Self::HAS_CURRENT_FRAME
    }

    pub fn has_sleep(&self) -> bool {
        self.0 & Self::HAS_SLEEP == Self::HAS_SLEEP
    }

    pub fn skip_frames(&self) -> bool {
        self.0 & Self::SKIP_FRAMES == Self::SKIP_FRAMES
    }

    pub fn has_depth_scale(&self) -> bool {
        self.0 & Self::HAS_DEPTH_SCALE == Self::HAS_DEPTH_SCALE
    }
}

#[derive(Debug)]
pub struct RenderMethod(u32);

impl RenderMethod {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    pub fn draw_style(&self) -> DrawStyle {
        // Safe to unwrap because mask limits to two bits
        // and all values are covered by the `DrawStyle` enum.
        FromPrimitive::from_u32(self.0 & 0b11).unwrap()
    }

    pub fn lighting(&self) -> Lighting {
        // Safe to unwrap because mask limits to three bits
        // and all values are covered by the `Lighting` enum.
        FromPrimitive::from_u32((self.0 >> 2) & 0b111).unwrap()
    }

    pub fn shading(&self) -> Shading {
        // Safe to unwrap because mask limits to two bits
        // and all values are covered by the `Shading` enum.
        FromPrimitive::from_u32((self.0 >> 5) & 0b11).unwrap()
    }

    pub fn texture_style(&self) -> TextureStyle {
        // Safe to unwrap because mask limits to four bits
        // and all values are covered by the `TextureStyle` enum.
        FromPrimitive::from_u32((self.0 >> 7) & 0b1111).unwrap()
    }

    pub fn unknown_bits(&self) -> u32 {
        (self.0 >> 11) & 0xfffff
    }

    pub fn user_defined(&self) -> bool {
        self.0 >> 31 == 1
    }
}

#[derive(FromPrimitive)]
pub enum DrawStyle {
    Transparent = 0x0,
    Unknown = 0x1,
    Wireframe = 0x2,
    Solid = 0x3,
}

#[derive(FromPrimitive)]
pub enum Lighting {
    ZeroIntensity = 0x0,
    Unknown1 = 0x1,
    Constant = 0x2,
    XXXXX = 0x3,
    Ambient = 0x4,
    ScaledAmbient = 0x5,
    Unknown2 = 0x6,
    Invalid = 0x7,
}

#[derive(FromPrimitive)]
pub enum Shading {
    None1 = 0x0,
    None2 = 0x1,
    Gouraud1 = 0x2,
    Gouraud2 = 0x3,
}

#[derive(FromPrimitive)]
pub enum TextureStyle {
    None = 0x0,
    XXXXXXXX1 = 0x1,
    Texture1 = 0x2,
    TransTexture1 = 0x3,
    Texture2 = 0x4,
    TransTexture2 = 0x5,
    Texture3 = 0x6,
    XXXXXXXX2 = 0x7,
    Texture4 = 0x8,
    TransTexture4 = 0x9,
    Texture5 = 0xa,
    TransTexture5 = 0xb,
    Unknown1 = 0xc,
    Unknown2 = 0xe,
    XXXXX = 0xf,
}

#[derive(Debug)]
pub struct RenderFlags(u32);

impl RenderFlags {
    const HAS_PEN: u32 = 0x01;
    const HAS_BRIGHTNESS: u32 = 0x02;
    const HAS_SCALED_AMBIENT: u32 = 0x04;
    const HAS_SIMPLE_SPRITE: u32 = 0x08;
    const HAS_UV_INFO: u32 = 0x10;
    // const UNKNOWN: u32 = 0x20;
    const IS_TWO_SIDED: u32 = 0x40;

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    pub fn has_pen(&self) -> bool {
        self.0 & Self::HAS_PEN == Self::HAS_PEN
    }

    pub fn has_brightness(&self) -> bool {
        self.0 & Self::HAS_BRIGHTNESS == Self::HAS_BRIGHTNESS
    }

    pub fn has_scaled_ambient(&self) -> bool {
        self.0 & Self::HAS_SCALED_AMBIENT == Self::HAS_SCALED_AMBIENT
    }

    pub fn has_simple_sprite(&self) -> bool {
        self.0 & Self::HAS_SIMPLE_SPRITE == Self::HAS_SIMPLE_SPRITE
    }

    pub fn has_uv_info(&self) -> bool {
        self.0 & Self::HAS_UV_INFO == Self::HAS_UV_INFO
    }

    pub fn is_two_sided(&self) -> bool {
        self.0 & Self::IS_TWO_SIDED == Self::IS_TWO_SIDED
    }
}

#[derive(Debug)]
/// `pitches` entries in the [TwoDimensionalObjectFragment]
pub struct SpritePitch {
    /// Windcatcher:
    /// _Unknown_ - Usually contains 0x200.
    ///
    /// NEW: Corresponds to PITCHCAP statement
    pub pitch_cap: i32,

    /// Windcatcher:
    /// The most significant bit of this field (0x80000000) is a flag
    /// of some sort. The other bits constitute another size field which
    /// we shall call `data6_size` here.
    /// NEW:
    /// Corresponds to NUMHEADINGS for a PITCH
    pub num_headings: u32,

    /// Windcatcher:
    /// There are `data6_size` of these.
    /// NEW:
    /// There are `num_headings` of these.
    pub headings: Vec<SpriteHeading>,
}

impl SpritePitch {
    fn parse(num_frames: u32, input: &[u8]) -> IResult<&[u8], SpritePitch> {
        let (i, (pitch_cap, num_headings)) = tuple((le_i32, le_u32))(input)?;
        let (remaining, headings) = count(
            |input| SpriteHeading::parse(num_frames, input),
            num_headings as usize,
        )(i)?;

        Ok((
            remaining,
            SpritePitch {
                pitch_cap,
                num_headings,
                headings,
            },
        ))
    }
}

#[derive(Debug)]
/// `headings` entries in [SpritePitch]
pub struct SpriteHeading {
    /// Windcatcher:
    /// _Unknown_ - Usually contains 64 (0x40).
    /// NEW:
    /// HEADINGCAP
    pub heading_cap: u32,

    /// These point to one or more 0x03 Texture Bitmap Name fragments
    /// (one if the object is static or more than one if it has an animated
    /// texture, such as blood from a weapon strike).
    /// There are `num_frames` of these.
    pub frames: Vec<u32>,
}

impl SpriteHeading {
    fn parse(num_frames: u32, input: &[u8]) -> IResult<&[u8], SpriteHeading> {
        let (remaining, (heading_cap, frames)) =
            tuple((le_u32, count(le_u32, num_frames as usize)))(input)?;
        Ok((
            remaining,
            SpriteHeading {
                heading_cap,
                frames,
            },
        ))
    }
}

#[derive(Debug)]
/// **Type ID:** 0x15
pub struct ObjectLocationFragment {
    /// Typically 0x2E when used in main zone files and 0x32E when
    /// used for placeable objects.
    pub flags: u32,

    /// When used in main zone files, points to a 0x16 fragment.
    /// When used for placeable objects, seems to always contain 0.
    /// This might be due to the difference in the Flags value.
    pub fragment1: u32,

    /// When used in main zone files, contains the minimum X value of the
    /// entire zone. When used for placeable objects, contains the X value
    /// of the object’s location.
    pub x: f32,

    /// When used in main zone files, contains the minimum Y value of the
    /// entire zone. When used for placeable objects, contains the Y value
    /// of the object’s location.
    pub y: f32,

    /// When used in main zone files, contains the minimum Z value of the
    /// entire zone. When used for placeable objects, contains the Z value
    /// of the object’s location.
    pub z: f32,

    /// When used in main zone files, typically contains 0. When used for
    /// placeable objects, contains a value describing rotation around the Z
    /// axis, scaled as Degrees x (512 / 360).
    pub rotate_z: f32,

    /// When used in main zone files, typically contains 0. When used for
    /// placeable objects, contains a value describing rotation around the Y
    /// axis, scaled as Degrees x (512 / 360).
    pub rotate_y: f32,

    /// When used in main zone files, typically contains 0. When used for
    /// placeable objects, contains a value describing rotation around the X
    /// axis, scaled as Degrees x (512 / 360).
    pub rotate_x: f32,

    /// _Unknown_ - Typically contains 0 (though might be more significant for placeable objects).
    pub params1: u32,

    /// When used in main zone files, typically contains 0.5. When used for
    /// placeable objects, contains the object’s scaling factor in the Y direction
    /// (e.g. 2.0 would make the object twice as big in the Y direction).
    pub scale_y: f32,

    /// When used in main zone files, typically contains 0.5. When used for
    /// placeable objects, contains the object’s scaling factor in the X direction
    /// (e.g. 2.0 would make the object twice as big in the X direction).
    pub scale_x: f32,

    /// When used in main zone files, typically contains 0 (might be related to
    /// the Flags value). When used for placeable objects, points to a 0x33 Vertex
    /// Color Reference fragment.
    pub fragment2: u32,

    /// Typically contains 30 when used in main zone files and 0 when used for
    /// placeable objects. This field only exists if `fragment2` points to a fragment.
    pub params2: u32,
}

impl FragmentType for ObjectLocationFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x15;

    fn parse(input: &[u8]) -> IResult<&[u8], ObjectLocationFragment> {
        let (
            remaining,
            (
                flags,
                fragment1,
                x,
                y,
                z,
                rotate_z,
                rotate_y,
                rotate_x,
                params1,
                scale_y,
                scale_x,
                fragment2,
                params2,
            ),
        ) = tuple((
            le_u32, le_u32, le_f32, le_f32, le_f32, le_f32, le_f32, le_f32, le_u32, le_f32, le_f32,
            le_u32, le_u32,
        ))(input)?;
        Ok((
            remaining,
            ObjectLocationFragment {
                flags,
                fragment1,
                x,
                y,
                z,
                rotate_z,
                rotate_y,
                rotate_x,
                params1,
                scale_y,
                scale_x,
                fragment2,
                params2,
            },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [MobSkeletonPieceTrackFragment].
///
/// **Type ID:** 0x13
pub struct MobSkeletonPieceTrackReferenceFragment {
    /// The [MobSkeletonPieceTrackFragment] reference.
    pub reference: FragmentRef<MobSkeletonPieceTrackFragment>,

    /// Most flags are _unknown_
    /// * bit 0 - If set `params1` exists.
    /// * bit 2 - Usually set.
    pub flags: u32,

    /// _Unknown_
    pub params1: Option<u32>,
}

impl FragmentType for MobSkeletonPieceTrackReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x13;

    fn parse(input: &[u8]) -> IResult<&[u8], MobSkeletonPieceTrackReferenceFragment> {
        let (i, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;

        let (remaining, params1) = if flags & 0x01 == 0x01 {
            le_u32(i).map(|(i, params1)| (i, Some(params1)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            MobSkeletonPieceTrackReferenceFragment {
                reference,
                flags,
                params1,
            },
        ))
    }
}

#[derive(Debug)]
/// ## Notes
/// This fragment describes how a skeleton piece is shifted or rotated relative to its parent
/// piece. The overall skeleton is contained in a 0x10 Skeleton Track Set fragment and is
/// structured as a hierarchical tree (see that fragment for information on how skeletons
/// are structured). The 0x12 fragment contains information on how that particular skeleton
/// piece is rotated and/or shifted relative to its parent piece.
///
/// Rotation and shifting information is contained as a series of fractions. The fragment
/// contains one denominator value for rotation and another for translation (X, Y, Z, shift).
/// It contains one numerator each for X, Y, Z rotation and shift, for a total of eight values.
/// For rotation, the resulting value should be multiplied by Pi / 2 radians (i.e. 1 corresponds
/// to 90 degrees, 2 corresponds to 180 degrees, etc.).
///
/// ## Fields
/// For rendering polygons, the X, Y, Z rotation and shift information in this fragment should
/// be taken into account by adding them to the rotation and shift values passed from the parent
/// piece (that is, rotation and shift are cumulative). However, before adding the shift values,
/// the X, Y, and Z shift values should first be rotated according to the parent’s rotation values.
/// The rotation values in this fragment represent the orientation of this piece relative to the
/// parent so calculating its starting position should not take its own rotation into account.
///
/// Software rendering a skeleton piece should perform the following steps in this order:
///   * Calculate the X, Y, and Z shift values from this fragment
///   * Rotate the shift values according to the rotation values from the parent piece
///   * Add the shift values to the shift values from the parent piece
///   * Calculate the X, Y, and Z rotation values from this fragment
///   * Add the rotation values to the rotation values from the parent piece
///   * Adjust the vertices for this piece by rotating them using the new rotation values and then
///     shifting them by the new shift values (or save the rotation and shift values for this piece
///     to be looked up later on when rendering)
///   * Process the next piece in the tree with the new rotation and shift values
///   * When all pieces have been processed, render all meshes in the model, using either the
///     adjusted vertex values (more efficient) or looking up the corresponding piece for each
///     vertex and adjusting the vertex values according to the adjusted rotation and shift values
///     calculated above (less efficient).
///
/// **Type ID:** 0x12
pub struct MobSkeletonPieceTrackFragment {
    /// Most flags are _unknown_.
    /// * bit 3 - If set then `data2` exists (though I’m not at all sure about this since I
    ///           have yet to see an example). It could instead mean that the rotation and
    ///           shift entries are `u32`s or it could mean that they’re `f32`s.
    pub flags: u32,

    /// The number of `data1` and `data2` entries there are.
    pub size: u32,

    /// This represents the denominator for the piece’s X, Y, and Z rotation values.
    /// It’s vital to note that it is possible to encounter situations where this value is zero.
    /// I have seen this for pieces with no vertices or polygons and in this case rotation should
    /// be ignored (just use the existing rotation value as passed from the parent piece). My belief
    /// is that such pieces represent attachment points for weapons or items (e.g. shields) and
    /// otherwise don’t represent a part of the model to be rendered.
    pub rotate_denominator: i16,

    /// The numerator for rotation about the X axis.
    pub rotate_x_numerator: i16,

    /// The numerator for rotation about the Y axis.
    pub rotate_y_numerator: i16,

    /// The numerator for rotation about the Z axis.
    pub rotate_z_numerator: i16,

    /// The numerator for translation along the X axis.
    pub shift_x_numerator: i16,

    /// The numerator for translation along the Y axis.
    pub shift_y_numerator: i16,

    /// The numerator for translation along the Z axis.
    pub shift_z_numerator: i16,

    /// The denominator for the piece X, Y, and Z shift values. Like the rotation denominator,
    /// software should check to see if this is zero and ignore translation in that case.
    pub shift_denominator: i16,

    /// _Unknown_ - There are (4 x Size) DWORDs here. This field exists only if the proper bit
    /// in Flags is set. It’s possible that this is a bogus field and really just represents
    /// the above fields in some sort of 32-bit form
    pub data2: Option<Vec<u8>>,
}

impl FragmentType for MobSkeletonPieceTrackFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x12;

    fn parse(input: &[u8]) -> IResult<&[u8], MobSkeletonPieceTrackFragment> {
        let (
            i,
            (
                flags,
                size,
                rotate_denominator,
                rotate_x_numerator,
                rotate_y_numerator,
                rotate_z_numerator,
                shift_x_numerator,
                shift_y_numerator,
                shift_z_numerator,
                shift_denominator,
            ),
        ) = tuple((
            le_u32, le_u32, le_i16, le_i16, le_i16, le_i16, le_i16, le_i16, le_i16, le_i16,
        ))(input)?;

        let (remaining, data2) = if i.len() > 0 && (flags & 0x08 == 0x08) {
            (&i[0..0], Some(i.to_vec()))
        } else {
            (i, None)
        };
        if remaining.len() > 0 {
            panic!(
                "Data2 of MobSkeletonPieceTrackFragment found - flags: {:?}, size: {:?}, len: {:?}, remaining: {:?}",
                flags, size, remaining.len(), remaining
            );
        }

        //let (remaining, data2) = if flags & 0x08 == 0x08 {
        //    count(le_i32, (size * 4) as usize)(i).map(|(i, data2)| (i, Some(data2)))?
        //} else {
        //    (i, None)
        //};

        Ok((
            remaining,
            MobSkeletonPieceTrackFragment {
                flags,
                size,
                rotate_denominator,
                rotate_x_numerator,
                rotate_y_numerator,
                rotate_z_numerator,
                shift_x_numerator,
                shift_y_numerator,
                shift_z_numerator,
                shift_denominator,
                data2,
            },
        ))
    }
}

#[derive(Debug)]
/// This fragment describes a skeleton for an entire animated model, and is used for mob
/// models. The overall skeleton is contained in a 0x10 [SkeletonTrackSetFragment] and
/// is structured as a hierarchical tree. For example, a pelvis piece might connect to chest,
/// left thigh, and right thigh pieces. The chest piece might connect to left bicep, right
/// bicep, and neck pieces. The left bicep piece might connect to a left forearm piece.
/// The left forearm piece might connect to a left hand piece. The idea is to start at the
/// base “stem” piece in the skeleton and recursively walk the tree to each successive piece.
///
/// For each piece there is a 0x13 [MobSkeletonPieceTrackReferenceFragment], which
/// references one 0x12 [MobSkeletonPieceTrackFragment]. Each 0x12 fragment defines
/// how that piece is rotated and/or shifted relative to its parent piece.
///
/// **Type ID:** 0x10
pub struct SkeletonTrackSetFragment {
    /// Most flags are _unknown_.
    /// * bit 0 - If set then `unknown_params1` exists.
    /// * bit 1 - If set then `unknown_params2` exists.
    /// * bit 9 - If set then `size2`, `fragment3`, and `data3` exist.
    pub flags: u32,

    /// The number of track reference entries
    pub entry_count: u32,

    /// Optionally points to a 0x18 [PolygonAnimationReferenceFragment]?
    pub fragment: u32,

    /// _Unknown_
    pub unknown_params1: Option<(u32, u32, u32)>,

    /// _Unknown_
    pub unknown_params2: Option<f32>,

    /// There are `entry_count` entries.
    pub entries: Vec<SkeletonTrackSetFragmentEntry>,

    /// The number of fragment3 and data3 entries there are.
    pub size2: Option<u32>,

    /// There are `size2` of these. This field only exists if the proper bit in the `flags`
    /// field is set. These entries generally point to 0x2D [MeshReferenceFragment]s and
    /// outline all of the meshes in the animated model. For example, there might be a mesh
    /// for a model’s body and another one for the head.
    pub fragment3: Option<Vec<u32>>,

    /// _Unknown_ - There are size2 of these.
    pub data3: Option<Vec<u32>>,
}

impl FragmentType for SkeletonTrackSetFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x10;

    fn parse(input: &[u8]) -> IResult<&[u8], SkeletonTrackSetFragment> {
        let (i, (flags, entry_count, fragment)) = tuple((le_u32, le_u32, le_u32))(input)?;

        let (i, unknown_params1) = if flags & 0x01 == 0x01 {
            tuple((le_u32, le_u32, le_u32))(i).map(|(i, p1)| (i, Some(p1)))?
        } else {
            (i, None)
        };

        let (i, unknown_params2) = if flags & 0x02 == 0x02 {
            le_f32(i).map(|(i, p2)| (i, Some(p2)))?
        } else {
            (i, None)
        };

        let (i, entries) = count(SkeletonTrackSetFragmentEntry::parse, entry_count as usize)(i)?;

        let (i, size2) = if flags & 0x200 == 0x200 {
            le_u32(i).map(|(i, size2)| (i, Some(size2)))?
        } else {
            (i, None)
        };

        let (remaining, (fragment3, data3)) = if flags & 0x200 == 0x200 {
            let size = size2.unwrap_or(0) as usize;
            tuple((count(le_u32, size), count(le_u32, size)))(i)
                .map(|(i, (f3, d3))| (i, (Some(f3), Some(d3))))?
        } else {
            (i, (None, None))
        };

        Ok((
            remaining,
            SkeletonTrackSetFragment {
                flags,
                entry_count,
                fragment,
                unknown_params1,
                unknown_params2,
                entries,
                size2,
                fragment3,
                data3,
            },
        ))
    }
}

#[derive(Debug)]
/// Entries in the map's [SkeletonTrackSetFragment]
pub struct SkeletonTrackSetFragmentEntry {
    /// This seems to refer to the name of either this or another 0x10 fragment.
    /// It seems that at least one name reference points to the name of this fragment.
    pub name_reference: u32,

    /// _Unknown_ - Usually 0x0
    pub flags: u32,

    /// Reference to a 0x13 Mob Skeleton Piece Track Reference fragment.
    ///
    /// Important: animated models generally only reference a basic set of fragments
    /// necessary to render the model but not animate it. There will generally be
    /// other sets of 0x13 fragments where each set corresponds to a different
    /// animation of the model. Software reading .WLD files must use the name of
    /// the first 0x13 fragment referenced by the 0x10 Skeleton Track Set to discover
    /// any other animation sets. The first fragment of any alternate animation set
    /// will have the same name as the first 0x13 fragment, with an additional prefix.
    /// All other 0x13 fragments in that same set will likewise correspond to their
    /// counterparts in the basic animation set. Different animation sets will have
    /// different prefixes (e.g. “C01” for one combat animation, “C02” for another
    /// combat animation, etc.). All alternate animation sets for a particular model
    /// generally immediately follow the 0x10 Skeleton Track Set fragment (with the
    /// 0x11 Skeleton Track Set Reference immediately following those). I don’t know
    /// if this is a necessary arrangement.
    pub fragment1: u32,

    /// Sometimes refers to a 0x2D Mesh Reference fragment.
    pub fragment2: u32,

    /// The number of data entries
    pub data_entry_count: u32,

    /// Each of these contains the index of the next piece in the skeleton tree. A
    /// Skeleton Track Set is a hierarchical tree of pieces in the skeleton. It
    /// generally starts with a central “stem” and branches out to a skeleton’s
    /// extremities. For instance, the first entry might be the stem; that entry
    /// might point to the pelvis entry; the pelvis entry might point to the left thigh,
    /// right thigh, and chest entries; and those entries would each point to other parts
    /// of the skeleton. The exact topography of the tree depends upon the overall
    /// structure of the skeleton. The proper way to use a Skeleton Track Set fragment
    /// is to start with the first entry and recursively walk the tree by following each
    /// entry’s Entry1Data field to other connected pieces.
    ///
    /// It’s also worth noting that, although an entry might reference a 0x13 Mob Skeleton
    /// Piece Track Reference fragment in its EntityFragment1 field, that does not mean it
    /// will be valid for rendering (see the 0x12 Mob Skeleton Piece Track fragment for more
    /// information). Many model skeletons apparently contain extraneous pieces that have an
    /// unknown purpose, though I suspect that they are for determining attachment points
    /// for weapons and shields and are otherwise not meant to be rendered. These pieces are
    /// generally not referenced by the 0x36 Mesh fragments that the skeleton indirectly
    /// references (via 0x2D Mesh Reference fragments).
    pub data_entries: Vec<u32>,
}

impl FragmentType for SkeletonTrackSetFragmentEntry {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], SkeletonTrackSetFragmentEntry> {
        let (i, (name_reference, flags, fragment1, fragment2, data_entry_count)) =
            tuple((le_u32, le_u32, le_u32, le_u32, le_u32))(input)?;

        let (remaining, data_entries) = count(le_u32, data_entry_count as usize)(i)?;

        Ok((
            remaining,
            SkeletonTrackSetFragmentEntry {
                name_reference,
                flags,
                fragment1,
                fragment2,
                data_entry_count,
                data_entries,
            },
        ))
    }
}

#[derive(Debug)]
/// Static or animated model reference or player info.
///
/// **Type ID:** 0x14
pub struct ModelFragment {
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
    pub name_fragment: u32,

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

impl FragmentType for ModelFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x14;

    fn parse(input: &[u8]) -> IResult<&[u8], ModelFragment> {
        let (i, (flags, name_fragment, unknown_params2_count, fragment_count, unknown_fragment)) =
            tuple((le_u32, le_u32, le_u32, le_u32, le_u32))(input)?;

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

#[derive(Debug)]
/// A map's BSP Tree.
///
/// **Type ID:** 0x21
pub struct BspTreeFragment {
    /// The number of [BspTreeFragmentEntry]s in this tree.
    pub size1: u32,

    /// The [BspTreeFragmentEntry]s
    pub entries: Vec<BspTreeFragmentEntry>,
}

impl FragmentType for BspTreeFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x21;

    fn parse(input: &[u8]) -> IResult<&[u8], BspTreeFragment> {
        let (i, size1) = le_u32(input)?;
        let (remaining, entries) = count(BspTreeFragmentEntry::parse, size1 as usize)(i)?;

        Ok((remaining, BspTreeFragment { size1, entries }))
    }
}

#[derive(Debug)]
/// Entries in the map's [BspTreeFragment]
pub struct BspTreeFragmentEntry {
    /// The normal to the split plane.
    pub normal: (f32, f32, f32),

    /// Distance from the split plane to the origin (0, 0, 0) in (x, y, z) space. With the above
    /// fields the splitting plane is represented in Hessian Normal Form.
    pub split_distance: f32,

    /// If this is a leaf node, this contains the index of the [BspRegionFragment] fragment that this
    /// refers to (with the lowest index being 1). Otherwise this will contain 0.
    pub region: FragmentRef<BspRegionFragment>,

    /// If this is not a leaf node these are references to [BspTreeFragmentEntry] on either side of the
    /// splitting plane.
    pub nodes: (
        FragmentRef<BspTreeFragmentEntry>,
        FragmentRef<BspTreeFragmentEntry>,
    ),
}

impl FragmentType for BspTreeFragmentEntry {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], BspTreeFragmentEntry> {
        let (remaining, (normal, split_distance, region, nodes)) = tuple((
            tuple((le_f32, le_f32, le_f32)),
            le_f32,
            fragment_ref,
            tuple((fragment_ref, fragment_ref)),
        ))(input)?;

        Ok((
            remaining,
            BspTreeFragmentEntry {
                normal,
                split_distance,
                region,
                nodes,
            },
        ))
    }
}

#[derive(Debug)]
/// A region within a map's BSP Tree.
///
/// **Type ID:** 0x22
pub struct BspRegionFragment {
    /// Most flags are _unknown_. Usually contains 0x181 for regions that contain polygons and 0x81
    /// for regions that are empty.
    /// * bit 5 - If set then `pvs` contains u32 entries.
    /// * bit 7 - If set then `pvs` contains u8 entries (more common).
    pub flags: u32,

    /// _Unknown_ - Some sort of fragment reference. Usually nothing is referenced.
    pub fragment1: FragmentRef<i32>,

    /// The number of bytes in `data1`
    pub size1: u32,

    /// The number of bytes in `data2`
    pub size2: u32,

    /// _Unknown_ - Usually 0
    pub params1: u32,

    /// The number of `data3` entries. Usually 0.
    pub size3: u32,

    /// The number of `data4` entries. Usually 0.
    pub size4: u32,

    /// _Unknown_ - Usually 0.
    pub params2: u32,

    /// The number of `data5` entries. Usually 1.
    pub size5: u32,

    /// The number of `pvs` entries. Usually 1.
    pub pvs_count: u32,

    /// According to the ZoneConverter source there are 12 * `size1` bytes here. Their format is
    /// _unknown_ for lack of sample data to figure it out.
    pub data1: Vec<u8>,

    /// According to the ZoneConverter source there are 8 * `size2` bytes here. Their format is
    /// _unknown_ for lack of sample data to figure it out.
    pub data2: Vec<u8>,

    /// _Unknown_ data entries
    pub data3: Vec<BspRegionFragmentData3Entry>,

    /// _Unknown_ data entries
    pub data4: Vec<BspRegionFragmentData4Entry>,

    /// _Unknown_ data entries
    pub data5: Vec<BspRegionFragmentData5Entry>,

    /// A potentially visible set (PVS) of regions
    pub pvs: Vec<BspRegionFragmentPVS>,

    /// The number of bytes in the `name7` field.
    pub size7: u32,

    /// _Unknown_ - An encoded string.
    pub name7: Vec<u8>,

    /// _Unknown_ - Usually references nothing.
    pub fragment2: FragmentRef<i32>,

    /// If there are any polygons in this region then this reference points to a [MeshFragment]
    /// that contains only those polygons. That [MeshFragment] must contain all geometry information
    /// contained within the volume that this region represents and nothing that lies outside of
    /// that volume.
    pub mesh_reference: Option<FragmentRef<MeshFragment>>,
}

impl FragmentType for BspRegionFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x22;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragment> {
        let (i, (flags, fragment1, size1, size2, params1, size3, size4, params2, size5, pvs_count)) =
            tuple((
                le_u32,
                fragment_ref,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
            ))(input)?;
        let (i, (data1, data2, data3, data4, data5, pvs, size7)) = tuple((
            count(le_u8, size1 as usize),
            count(le_u8, size2 as usize),
            count(BspRegionFragmentData3Entry::parse, size3 as usize),
            count(BspRegionFragmentData4Entry::parse, size4 as usize),
            count(BspRegionFragmentData5Entry::parse, size5 as usize),
            count(BspRegionFragmentPVS::parse, pvs_count as usize),
            le_u32,
        ))(i)?;
        let (i, (name7, fragment2)) = tuple((count(le_u8, 12), fragment_ref))(i)?;

        let (remaining, mesh_reference) = if (flags & 0x100) == 0x100 {
            fragment_ref(i).map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            BspRegionFragment {
                flags,
                fragment1,
                size1,
                size2,
                params1,
                size3,
                size4,
                params2,
                size5,
                pvs_count,
                data1,
                data2,
                data3,
                data4,
                data5,
                pvs,
                size7,
                name7,
                fragment2,
                mesh_reference,
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct BspRegionFragmentData3Entry {
    /// _Unknown_
    /// * bit 1 - If set then the `params1`and `params2` fields exist.
    flags: u32,

    /// The number of `data1` entries.
    size1: u32,

    /// _Unknown_
    data1: Vec<u32>,

    /// _Unknown_ - Only exists if bit 1 of `flags` is set.
    params1: Option<(u32, u32, u32)>,

    /// _Unknown_ - Only exists if bit 1 of `flags` is set.
    params2: Option<u32>,
}

impl FragmentType for BspRegionFragmentData3Entry {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentData3Entry> {
        let (i, (flags, size1)) = tuple((le_u32, le_u32))(input)?;
        let (i, data1) = count(le_u32, size1 as usize)(i)?;

        let has_params = flags & 0x02 == 0x02;
        let (remaining, (params1, params2)) = if has_params {
            tuple((
                map(tuple((le_u32, le_u32, le_u32)), Some),
                map(le_u32, Some),
            ))(i)?
        } else {
            (i, (None, None))
        };

        Ok((
            remaining,
            BspRegionFragmentData3Entry {
                flags,
                size1,
                data1,
                params1,
                params2,
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct BspRegionFragmentData4Entry {
    /// _Unknown_
    flags: u32,

    /// _Unknown_
    params1: u32,

    /// _Unknown_ - This seems to determine if `params2a` and/or `params2b` exist.
    type_field: u32,

    /// _Unknown_ - Only exists if `type_field` is greater than 7.
    params2a: Option<u32>,

    /// _Unknown_ - Only exists if `type_field` is one of the following:
    /// * 0x0A
    /// * 0x0B
    /// * 0x0C
    /// Though I'm not at all sure about this due to lack of sample data.
    params2b: Option<u32>,

    /// The number of bytes in the `name` field.
    name_size: u32,

    /// An encoded string.
    name: String,
}

impl FragmentType for BspRegionFragmentData4Entry {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentData4Entry> {
        let (i, (flags, params1, type_field)) = tuple((le_u32, le_u32, le_u32))(input)?;

        let (i, params2a) = if type_field > 7 {
            map(le_u32, Some)(i)?
        } else {
            (i, None)
        };

        let (i, params2b) = if type_field > 7 {
            map(le_u32, Some)(i)?
        } else {
            (i, None)
        };

        let (i, name_size) = le_u32(i)?;

        let (remaining, name) = count(le_u8, name_size as usize)(i)?;

        Ok((
            remaining,
            BspRegionFragmentData4Entry {
                flags,
                params1,
                type_field,
                params2a,
                params2b,
                name_size,
                name: String::from_utf8(name).unwrap(),
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct BspRegionFragmentData5Entry {
    /// _Unknown_ - Usually 0.
    params1: (u32, u32, u32),

    /// _Unknown_ - Usually 0.
    params2: u32,

    /// _Unknown_ - Usually 1.
    params3: u32,

    /// _Unknown_ - Usually 0.
    params4: u32,

    /// _Unknown_ - Usually 0.
    params5: u32,
}

impl FragmentType for BspRegionFragmentData5Entry {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentData5Entry> {
        let (remaining, (params1, params2, params3, params4, params5)) = tuple((
            tuple((le_u32, le_u32, le_u32)),
            le_u32,
            le_u32,
            le_u32,
            le_u32,
        ))(input)?;

        Ok((
            remaining,
            BspRegionFragmentData5Entry {
                params1,
                params2,
                params3,
                params4,
                params5,
            },
        ))
    }
}

#[derive(Debug)]
/// A potentially visible set (PVS) of regions
pub struct BspRegionFragmentPVS {
    /// The number of entries in the `data` field
    size: u16,

    /// This is a complicated field. It contains run-length-encoded data that tells the
    /// client which regions are “nearby”. The purpose appears to be so that the client
    /// can determine which mobs in the zone have to have their Z coordinates checked,
    /// so that they will fall to the ground (or until they land on something). Since
    /// it’s expensive to do this, it makes sense to only do it for regions that are
    /// visible to the player instead of doing it for all mobs in the entire zone (repeatedly).
    ///
    /// I’ve only encountered data where the stream is a list of BYTEs instead of WORDs.
    /// The following discussion describes RLE encoding a BYTE stream.
    ///
    /// The idea here is to form a sorted list of all region IDs that are within a
    /// certain distance, and then write that list as an RLE-encoded stream to save space.
    /// The procedure is as follows:
    ///
    /// 1. Set an initial region ID value to zero.
    /// 2. If this region ID is not present in the (sorted) list, skip forward to the first
    ///    one that is in the list. Write something to the stream that tells it how many IDs
    ///    were skipped.
    /// 3. Form a block of consecutive IDs that are in the list and write something to the
    ///    stream that tells the client that there are this many IDs that are in the list.
    /// 4. If there are more region IDs in the list, go back to step 2.
    ///
    /// When writing to the stream, either one or three bytes are written:
    ///
    /// * 0x00..0x3E - skip forward by this many region IDs
    /// * 0x3F, WORD - skip forward by the amount given in the following 16-bit WORD
    /// * 0x40..0x7F - skip forward based on bits 3..5, then include the number of
    ///                IDs based on bits 0..2
    /// * 0x80..0xBF - include the number of IDs based on bits 3..5, then skip forward
    ///                based on bits 0..2
    /// * 0xC0..0xFE - subtracting 0xC0, this many region IDs are nearby
    /// * 0xFF, WORD - the number of region IDs given by the following WORD are nearby
    ///
    /// It should be noted that the values in the range 0x40..0xBF allow skipping and
    /// including of no more than seven IDs at a time. Also, they are not necessary to
    /// encode a region list: they merely allow better compression.
    data: Vec<u8>,
}

impl FragmentType for BspRegionFragmentPVS {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentPVS> {
        let (i, size) = le_u16(input)?;
        let (remaining, data) = count(le_u8, size as usize)(i)?;

        Ok((remaining, BspRegionFragmentPVS { size, data }))
    }
}

#[derive(Debug)]
/// This is the fragment most often used for models. However, [AlternateMeshFragment] fragment
/// is also sometimes used.
///
/// **Type ID:** 0x36
pub struct MeshFragment {
    /// _Unknown_ - The meaning of the flags is unknown but the following values
    /// have been observed:
    ///
    /// * For zone meshes: 0x00018003
    /// * For placeable objects: 0x00014003
    pub flags: u32,

    /// A reference to a [MaterialListFragment] fragment. This tells the client which materials
    /// this mesh uses.
    ///
    /// For zone meshes the [MaterialListFragment] contains all the materials used in the
    /// entire zone.
    ///
    /// For placeable objects the [MaterialListFragment] contains all of the materials used in
    /// that object.
    pub material_list_ref: FragmentRef<MaterialListFragment>,

    /// A reference to a [MeshAnimatedVerticesReferenceFragment]. This is set for non-character
    /// animated meshes. For example swaying flags and trees.
    pub animation_ref: FragmentRef<i32>,

    /// _Unknown_ - Usually empty
    pub fragment3: FragmentRef<i32>,

    /// _Unknown_ - This usually seems to reference the first [TextureImagesFragment] fragment in the file.
    pub fragment4: FragmentRef<i32>,

    /// For zone meshes this typically contains the X coordinate of the center of the mesh.
    /// This allows vertex coordinates in the mesh to be relative to the center instead of
    /// having absolute coordinates. This is important for preserving precision when encoding
    /// vertex coordinate values.
    ///
    /// For placeable objects this seems to define where the vertices will lie relative to
    /// the object’s local origin. This seems to allow placeable objects to be created that
    /// lie at some distance from their position as given in a [ObjectLocationFragment]
    /// (why one would do this is a mystery, though).
    pub center: (f32, f32, f32),

    /// _Unknown_ - Usually (0, 0, 0).
    pub params2: (u32, u32, u32),

    /// Given the values in `center`, this seems to contain the maximum distance between any
    /// vertex and that position. It seems to define a radius from that position within which
    /// the mesh lies.
    pub max_distance: f32,

    /// Contains min x, y, and z coords in absolute coords of any vertex in the mesh.
    pub min: (f32, f32, f32),

    /// Contains max x, y, and z coords in absolute coords of any vertex in the mesh.
    pub max: (f32, f32, f32),

    /// Tells how many vertices there are in the mesh. Normally this is three times
    /// the number of polygons, but this is by no means necessary as polygons can
    /// share vertices. However, sharing vertices degrades the ability to use vertex
    /// normals to make a mesh look more rounded (with shading).
    pub position_count: u16,

    /// The number of texture coordinate pairs there are in the mesh. This should
    /// equal the number of vertices in the mesh. Presumably this could contain zero
    /// if none of the polygons have textures mapped to them (but why would anyone do that?)
    pub texture_coordinate_count: u16,

    /// The number of vertex normal entries in the mesh. This should equal the number
    /// of vertices in the mesh. Presumably this could contain zero if vertices should
    /// use polygon normals instead, but I haven’t tried it (vertex normals are preferable
    /// anyway).
    pub normal_count: u16,

    /// The number of vertex color entries in the mesh. This should equal the number
    /// of vertices in the mesh, or zero if there are no vertex color entries.
    /// Meshes do not require color entries to work. Color entries are used for
    /// illuminating polygons when there is a nearby light source.
    pub color_count: u16,

    /// The number of polygons in the mesh.
    pub polygon_count: u16,

    /// This seems to only be used when dealing with animated (mob) models.
    /// It contains the number of vertex piece entries. Vertices are grouped together by
    /// skeleton piece in this case and vertex piece entries tell the client how
    /// many vertices are in each piece. It’s possible that there could be more
    /// pieces in the skeleton than are in the meshes it references. Extra pieces have
    /// no polygons or vertices and I suspect they are there to define attachment points for
    /// objects (e.g. weapons or shields).
    pub vertex_piece_count: u16,

    /// The number of polygon texture entries. Polygons are grouped together by
    /// material and polygon material entries. This tells the client the number of
    /// polygons using a material.
    pub polygon_material_count: u16,

    /// The number of vertex material entries. Vertices are grouped together
    /// by material and vertex material entries tell the client how many vertices there
    /// are using a material.
    pub vertex_material_count: u16,

    /// _Unknown_ - The number of entries in `data9`. Seems to be used only for
    /// animated mob models.
    pub size9: u16,

    /// This allows vertex coordinates to be stored as integral values instead of
    /// floating-point values, without losing precision based on mesh size. Vertex
    /// values are multiplied by (1 shl `scale`) and stored in the vertex entries.
    pub scale: u16,

    /// Vertices (x, y, z) belonging to this mesh. Each axis should
    /// be multiplied by (1 shl `scale`) for the final vertex position.
    pub positions: Vec<(i16, i16, i16)>,

    /// Texture coordinates (x, y) used to map textures to this mesh.
    ///
    /// Two formats are possible:
    /// * Old - Signed 16-bit texture value in pixels (most textures are 256 pixels in size).
    /// * New - Signed 32-bit value
    pub texture_coordinates: Vec<(i16, i16)>,

    /// Vertex normals (x, y, z). Each element contains a signed byte representing the
    /// component of the vertex normal, scaled such that –127 represents –1 and
    /// 127 represents 1.
    pub vertex_normals: Vec<(i8, i8, i8)>,

    /// This contains an RGBA color value for each vertex in the mesh.
    /// It specifies the additional color to be applied to the vertex, as
    /// if that vertex has been illuminated by a nearby light source. The A value
    /// isn’t fully understood; I believe it represents an alpha as applied to
    /// the texture, such that 0 makes the polygon a pure color and 0xFF either
    /// illuminates an unaltered texture or mutes the illumination completely.
    /// That is, it’s either a blending value or an alpha value. Further
    /// experimentation is required. 0xD9 seems to be a good (typical) A value for
    /// most illuminated vertices.
    pub vertex_colors: Vec<u32>,

    /// A collection of [MeshFragmentPolygonEntry]s used in this mesh.
    pub polygons: Vec<MeshFragmentPolygonEntry>,

    /// The first element of the tuple is the number of vertices in a skeleton piece.
    ///
    /// The second element of the tuple is the index of the piece according to the
    /// [SkeletonTrackSet] fragment. The very first piece (index 0) is usually not referenced here
    /// as it is usually jsut a "stem" starting point for the skeleton. Only those pieces
    /// referenced here in the mesh should actually be rendered. Any other pieces in the skeleton
    /// contain no vertices or polygons And have other purposes.
    pub vertex_pieces: Vec<(u16, u16)>,

    /// The first element of the tuple is the number of polygons that use the same material. All
    /// polygon entries are sorted by material index so that polygons use the same material are
    /// grouped together.
    ///
    /// The second element of the tuple is the index of the material that the polygons use according
    /// to the [MaterialListFragment] that this fragment references.
    pub polygon_materials: Vec<(u16, u16)>,

    /// The first element of the tuple is the number of vertices that use the same
    /// material. Vertex materials, like polygons, are sorted by material index so
    /// that vertices that use the same material are together.
    ///
    /// The second element of the tuple is the index of the material that the
    /// vertices use, according to the [MaterialListFragment] fragment that this fragment
    /// references.
    pub vertex_materials: Vec<(u16, u16)>,

    /// _Unknown_ - A collection of [MeshFragmentData9Entry]s
    pub data9: Vec<MeshFragmentData9Entry>,
}

impl FragmentType for MeshFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x36;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshFragment> {
        let (
            i,
            (
                flags,
                material_list_ref,
                animation_ref,
                fragment3,
                fragment4,
                center,
                params2,
                max_distance,
                min,
                max,
                position_count,
                texture_coordinate_count,
                normal_count,
                color_count,
                polygon_count,
                vertex_piece_count,
                polygon_material_count,
                vertex_material_count,
                size9,
                scale,
            ),
        ) = tuple((
            le_u32,
            fragment_ref,
            fragment_ref,
            fragment_ref,
            fragment_ref,
            tuple((le_f32, le_f32, le_f32)),
            tuple((le_u32, le_u32, le_u32)),
            le_f32,
            tuple((le_f32, le_f32, le_f32)),
            tuple((le_f32, le_f32, le_f32)),
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
        ))(input)?;

        let (
            remaining,
            (
                positions,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                polygons,
                vertex_pieces,
                polygon_materials,
                vertex_materials,
                data9,
            ),
        ) = tuple((
            count(tuple((le_i16, le_i16, le_i16)), position_count as usize),
            count(tuple((le_i16, le_i16)), texture_coordinate_count as usize),
            count(tuple((le_i8, le_i8, le_i8)), normal_count as usize),
            count(le_u32, color_count as usize),
            count(MeshFragmentPolygonEntry::parse, polygon_count as usize),
            count(tuple((le_u16, le_u16)), vertex_piece_count as usize),
            count(tuple((le_u16, le_u16)), polygon_material_count as usize),
            count(tuple((le_u16, le_u16)), vertex_material_count as usize),
            count(MeshFragmentData9Entry::parse, size9 as usize),
        ))(i)?;

        Ok((
            remaining,
            MeshFragment {
                flags,
                material_list_ref,
                animation_ref,
                fragment3,
                fragment4,
                center,
                params2,
                max_distance,
                min,
                max,
                position_count,
                texture_coordinate_count,
                normal_count,
                color_count,
                polygon_count,
                vertex_piece_count,
                polygon_material_count,
                vertex_material_count,
                size9,
                scale,
                positions,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                polygons,
                vertex_pieces,
                polygon_materials,
                vertex_materials,
                data9,
            },
        ))
    }
}

#[derive(Debug)]
/// Represents a polygon within a [MeshFragment].
pub struct MeshFragmentPolygonEntry {
    /// Most flags are _Unknown_. This usually contains 0x0 for polygons but
    /// contains 0x0010 for polygons that the player can pass through (like water
    /// and tree leaves).
    flags: u16,

    /// An index for each of the polygon's vertex coordinates (idx1, idx2, idx3).
    pub vertex_indexes: (u16, u16, u16),
}

impl FragmentType for MeshFragmentPolygonEntry {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshFragmentPolygonEntry> {
        let (remaining, (flags, vertex_indexes)) =
            tuple((le_u16, tuple((le_u16, le_u16, le_u16))))(input)?;
        Ok((
            remaining,
            MeshFragmentPolygonEntry {
                flags,
                vertex_indexes,
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct MeshFragmentData9Entry {
    /// _Unknown_ - This seems to reference one of the vertex entries. This field
    /// only exists if `type_field` contains a value in the range 1-3.
    pub index1: Option<u16>,

    /// _Unknown_ - This seems to reference one of the vertex entries. This field is only valid if
    /// `type_field` contains 1. Otherwise, this field must contain 0.
    pub index2: Option<u16>,

    /// _Unknown_ - If `type_field` contains 4, then this field exists instead of `index1`
    /// and `index2`. [MeshFragmentData9Entry]s seem to be sorted by this value.
    pub offset: Option<f32>,

    /// _Unknown_ - It seems to only contain values in the range 0-2.
    pub param1: u16,

    /// _Unknown_ - It seems to control whether `index1`, `index2`, and `offset` exist. It can only
    /// contain values in the range 1-4. It looks like the [MeshFragmentData9Entry]s are broken up into
    /// blocks, where each block is terminated by an entry where `type_field` is 4.
    pub type_field: u16,
}

impl FragmentType for MeshFragmentData9Entry {
    type T = Self;

    const TYPE_ID: u32 = 0x00;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshFragmentData9Entry> {
        let (remaining, (index1, index2, offset, param1, type_field)) = tuple((
            map(le_u16, Some),
            map(le_u16, Some),
            map(le_f32, Some),
            le_u16,
            le_u16,
        ))(input)?;
        Ok((
            remaining,
            MeshFragmentData9Entry {
                index1,
                index2,
                offset,
                param1,
                type_field,
            },
        ))
    }
}

#[derive(Debug)]
///
/// **Type ID:** 0x31
pub struct MaterialListFragment {
    /// _Unknown_ - Must contain 0.
    pub flags: u32,

    /// The number of fragment references this fragment contains.
    pub size1: u32,

    /// `size1` references to [MaterialFragment] fragments.
    pub fragments: Vec<FragmentRef<MaterialFragment>>,
}

impl FragmentType for MaterialListFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x31;

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialListFragment> {
        let (i, (flags, size1)) = tuple((le_u32, le_u32))(input)?;
        let (remaining, fragments) = count(fragment_ref, size1 as usize)(i)?;
        Ok((
            remaining,
            MaterialListFragment {
                flags,
                size1,
                fragments,
            },
        ))
    }
}

#[derive(Debug)]
///
/// **Type ID:** 0x30
pub struct MaterialFragment {
    /// Most flags are _unknown_, however:
    /// * bit 1 - If set then the `pair` field exists. This is usually set.
    pub flags: u32,

    /// Most flags are _unknown_, however:
    /// * bit 0 - It seems like this must be set if the texture is not transparent.
    /// * bit 1 - Set if the texture is masked (e.g. tree leaves).
    /// * bit 2 - Set if the texture is semi-transparent but not masked.
    /// * bit 3 - Set if the texture is masked and semi-transparent.
    /// * bit 4 Set if the texture is masked but not semi-transparent.
    /// * bit 31 - It seems like this must be set if the texture is not transparent.
    pub params1: u32,

    /// This typically contains 0x004E4E4E but has also bee known to contain 0xB2B2B2.
    /// Could this be an RGB reflectivity value?
    pub params2: u32,

    /// _Unknown_ - Usually contains 0.
    pub params3: (f32, f32),

    /// A reference to a [TextureReferenceFragment] fragment.
    pub reference: FragmentRef<TextureReferenceFragment>,

    /// _Unknown_ - This only exists if bit 1 of flags is set. Both fields usually contain 0.
    pub pair: Option<(u32, f32)>,
}

impl FragmentType for MaterialFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x30;

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialFragment> {
        let (i, (flags, params1, params2, params3, reference)) = tuple((
            le_u32,
            le_u32,
            le_u32,
            tuple((le_f32, le_f32)),
            fragment_ref,
        ))(input)?;

        let (remaining, pair) = if flags & 0x2 == 0x2 {
            tuple((le_u32, le_f32))(i).map(|(rem, p)| (rem, Some(p)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            MaterialFragment {
                flags,
                params1,
                params2,
                params3,
                reference,
                pair,
            },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [TextureFragment] fragment.
///
/// **Type ID:** 0x05
pub struct TextureReferenceFragment {
    /// The [TextureFragment] reference.
    pub reference: FragmentRef<TextureFragment>,

    /// _Unknown_ - Seems to always contain 0x50.
    pub flags: u32,
}

impl FragmentType for TextureReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x05;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((remaining, TextureReferenceFragment { reference, flags }))
    }
}

#[derive(Debug)]
/// A reference to a [MeshFragment] fragment.
///
/// **Type ID:** 0x2d
pub struct MeshReferenceFragment {
    /// The [MeshFragment] reference.
    pub reference: FragmentRef<MeshFragment>,

    /// _Unknown_ - Apparently must be zero.
    pub params: u32,
}

impl FragmentType for MeshReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2d;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshReferenceFragment> {
        let (remaining, (reference, params)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((remaining, MeshReferenceFragment { reference, params }))
    }
}

#[derive(Debug)]
/// This fragment represents an entire texture rather than merely a bitmap used by that
/// texture. The conceptual difference from [TextureImagesFragment] fragments is that textures
/// may be animated; the [TextureFragment] fragment represents the entire texture
/// including all bitmaps that it uses whereas a [TextureImagesFragment] fragment would
/// represent only a single bitmap in the animated sequence.
///
/// **Type ID:** 0x04
pub struct TextureFragment {
    /// Most flags are _unknown_ however:
    /// * bit 3 - If set texture is animated (has more than one [TextureImagesFragment] reference.
    /// This also means that a `params1` field exists.
    /// * bit 4 - If set a `params2` field exists. This _seems_ to always be set.
    pub flags: TextureFragmentFlags,

    /// The number of [TextureImagesFragment] references.
    pub frame_count: u32,

    /// Only present if bit `has_current_frame` in `flags` is set.
    pub current_frame: Option<u32>,

    /// Only present if `sleep` in `flags` is set.
    pub sleep: Option<u32>,

    /// One or more references to [TextureImagesFragment] fragments. For most textures this will
    /// be a single reference but animated textures will reference multiple.
    pub frame_references: Vec<FragmentRef<TextureImagesFragment>>,
}

impl FragmentType for TextureFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x04;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureFragment> {
        let (i, (flags, frame_count)) = tuple((TextureFragmentFlags::parse, le_u32))(input)?;

        // TODO: Do these fields even really exist?
        let current_frame = None;
        let sleep = None;
        let (remaining, frame_references) = count(fragment_ref, frame_count as usize)(i)?;

        Ok((
            remaining,
            TextureFragment {
                flags,
                frame_count,
                current_frame,
                sleep,
                frame_references,
            },
        ))
    }
}

#[derive(Debug)]
pub struct TextureFragmentFlags(pub u32);

impl TextureFragmentFlags {
    const SKIP_FRAMES: u32 = 0x02;
    const IS_ANIMATED: u32 = 0x08;
    const HAS_SLEEP: u32 = 0x10;
    const HAS_CURRENT_FRAME: u32 = 0x10;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureFragmentFlags> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, TextureFragmentFlags(raw_flags)))
    }

    pub fn skip_frames(&self) -> bool {
        self.0 & Self::SKIP_FRAMES == Self::SKIP_FRAMES
    }

    pub fn is_animated(&self) -> bool {
        self.0 & Self::IS_ANIMATED == Self::IS_ANIMATED
    }

    pub fn has_sleep(&self) -> bool {
        self.0 & Self::HAS_SLEEP == Self::HAS_SLEEP
    }

    pub fn has_current_frame(&self) -> bool {
        self.0 & Self::HAS_CURRENT_FRAME == Self::HAS_CURRENT_FRAME
    }
}

#[derive(Debug)]
/// This fragment references one or more texture filenames. So far all known textures
/// reference a single filename.
///
/// **Type ID:** 0x03
pub struct TextureImagesFragment {
    /// Contains the number of texture filenames in this fragment. Again, this appears
    /// to always be 1.
    pub size1: u32,

    /// Bitmap filename entries
    pub entries: Vec<TextureImagesFragmentEntry>,
}

impl FragmentType for TextureImagesFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x03;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureImagesFragment> {
        let (i, size1) = le_u32(input)?;
        // TODO: This is hardcoded to one entry, is this all we need?
        let (remaining, entries) = count(TextureImagesFragmentEntry::parse, 1 as usize)(i)?;
        Ok((remaining, TextureImagesFragment { size1, entries }))
    }
}

#[derive(Debug)]
/// Bitmap filename entries within the [TextureImagesFragment] fragment.
pub struct TextureImagesFragmentEntry {
    /// The length of the filename in bytes.
    pub name_length: u16,

    /// The encoded filename. See [string hash encoding].
    ///
    /// The client apparently looks for certain filenames and substitutes built-in
    /// textures in their place. When using an animated fire texture where the names
    /// are fire1.bmp, fire2.bmp, fire3.bmp and fire4.bmp, respectively, the client always
    /// uses its built-in fire textures instead. This only happens when the textures are
    /// used by a placeable object and not when the textures are in the main zone file.
    /// It is unknown whether the substitution depends on the presence and exact order
    /// of all four textures.
    pub file_name: String,
}

impl FragmentType for TextureImagesFragmentEntry {
    type T = Self;

    const TYPE_ID: u32 = 0x0;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureImagesFragmentEntry> {
        let (i, name_length) = le_u16(input)?;
        let (remaining, file_name) = count(le_u8, name_length as usize)(i)?;
        Ok((
            remaining,
            TextureImagesFragmentEntry {
                name_length,
                file_name: decode_string(&file_name),
            },
        ))
    }
}
