use std::any::Any;

use super::{Fragment, FragmentType};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

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

impl Fragment for TwoDimensionalObjectFragment {
    fn serialize(&self) -> Vec<u8> {
        vec![].iter().flatten().collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
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
