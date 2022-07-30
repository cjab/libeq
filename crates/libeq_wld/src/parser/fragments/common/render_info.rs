use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Represents a polygon within a [MeshFragment].
pub struct RenderInfo {
    pub flags: RenderInfoFlags,

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
    pub simple_sprite_reference: Option<u32>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 4 of `renderinfo_flags` is set.
    /// It looks like some sort of transformation matrix.
    /// NEW:
    /// Corresponds to UVORIGIN, UAXIS, and VAXIS in RENDERINFO
    pub uv_info: Option<UvInfo>,

    /// Windcatcher:
    /// _Unknown_ - Only exists if bit 5 of `renderinfo_flags` is set.
    /// NEW:
    /// Corresponds to UV entries in RENDERINFO
    pub uv_map: Option<UvMap>,
}

impl RenderInfo {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, flags) = RenderInfoFlags::parse(input)?;
        let (i, pen) = if flags.has_pen() {
            le_u32(i).map(|(i, p2)| (i, Some(p2)))?
        } else {
            (i, None)
        };
        let (i, brightness) = if flags.has_brightness() {
            le_f32(i).map(|(i, p3)| (i, Some(p3)))?
        } else {
            (i, None)
        };
        let (i, scaled_ambient) = if flags.has_scaled_ambient() {
            le_f32(i).map(|(i, p4)| (i, Some(p4)))?
        } else {
            (i, None)
        };
        let (i, simple_sprite_reference) = if flags.has_simple_sprite() {
            le_u32(i).map(|(i, f)| (i, Some(f)))?
        } else {
            (i, None)
        };
        let (i, uv_info) = if flags.has_uv_info() {
            UvInfo::parse(i).map(|(i, m)| (i, Some(m)))?
        } else {
            (i, None)
        };
        let (i, uv_map) = if flags.has_uv_map() {
            UvMap::parse(i).map(|(i, s)| (i, Some(s)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            Self {
                flags,
                pen,
                brightness,
                scaled_ambient,
                simple_sprite_reference,
                uv_info,
                uv_map,
            },
        ))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.flags.into_bytes()[..],
            &self.pen.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.brightness.map_or(vec![], |b| b.to_le_bytes().to_vec())[..],
            &self
                .scaled_ambient
                .map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
            &self
                .simple_sprite_reference
                .map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.uv_info.as_ref().map_or(vec![], |u| u.into_bytes())[..],
            &self.uv_map.as_ref().map_or(vec![], |u| u.into_bytes())[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct RenderInfoFlags(u32);

impl RenderInfoFlags {
    const HAS_PEN: u32 = 0x01;
    const HAS_BRIGHTNESS: u32 = 0x02;
    const HAS_SCALED_AMBIENT: u32 = 0x04;
    const HAS_SIMPLE_SPRITE: u32 = 0x08;
    const HAS_UV_INFO: u32 = 0x10;
    const HAS_UV_MAP: u32 = 0x20;
    const IS_TWO_SIDED: u32 = 0x40;

    pub fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
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

    pub fn has_uv_map(&self) -> bool {
        self.0 & Self::HAS_UV_MAP == Self::HAS_UV_MAP
    }

    pub fn is_two_sided(&self) -> bool {
        self.0 & Self::IS_TWO_SIDED == Self::IS_TWO_SIDED
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct UvInfo {
    pub uv_origin: (f32, f32, f32),
    pub u_axis: (f32, f32, f32),
    pub v_axis: (f32, f32, f32),
}

impl UvInfo {
    fn parse(input: &[u8]) -> IResult<&[u8], UvInfo> {
        let (i, uv_origin) = tuple((le_f32, le_f32, le_f32))(input)?;
        let (i, u_axis) = tuple((le_f32, le_f32, le_f32))(i)?;
        let (i, v_axis) = tuple((le_f32, le_f32, le_f32))(i)?;

        Ok((
            i,
            UvInfo {
                uv_origin,
                u_axis,
                v_axis,
            },
        ))
    }

    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.uv_origin.0.to_le_bytes()[..],
            &self.uv_origin.1.to_le_bytes()[..],
            &self.uv_origin.2.to_le_bytes()[..],
            &self.u_axis.0.to_le_bytes()[..],
            &self.u_axis.1.to_le_bytes()[..],
            &self.u_axis.2.to_le_bytes()[..],
            &self.v_axis.0.to_le_bytes()[..],
            &self.v_axis.1.to_le_bytes()[..],
            &self.v_axis.2.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct UvMap {
    pub entry_count: u32,
    pub entries: Vec<(f32, f32)>,
}

impl UvMap {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, entry_count) = le_u32(input)?;
        let (i, entries) = count(tuple((le_f32, le_f32)), entry_count as usize)(i)?;

        Ok((
            i,
            UvMap {
                entry_count,
                entries,
            },
        ))
    }

    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.entry_count.to_le_bytes()[..],
            &self
                .entries
                .iter()
                .flat_map(|(u, v)| [u.to_le_bytes(), v.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct RenderMethod(u32);

impl RenderMethod {
    pub fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum DrawStyle {
    Transparent = 0x0,
    Unknown = 0x1,
    Wireframe = 0x2,
    Solid = 0x3,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, PartialEq)]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum Shading {
    None1 = 0x0,
    None2 = 0x1,
    Gouraud1 = 0x2,
    Gouraud2 = 0x3,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, PartialEq)]
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
