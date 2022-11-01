use super::WResult;

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

use num_derive::{FromPrimitive, ToPrimitive};
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
    pub fn parse(input: &[u8]) -> WResult<Self> {
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

    pub fn parse(input: &[u8]) -> WResult<Self> {
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
    fn parse(input: &[u8]) -> WResult<UvInfo> {
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
    fn parse(input: &[u8]) -> WResult<Self> {
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
#[derive(PartialEq)]
pub enum RenderMethod {
    Standard {
        draw_style: DrawStyle,
        lighting: Lighting,
        shading: Shading,
        texture_style: TextureStyle,
        unknown_bits: u32,
    },
    UserDefined {
        material_type: MaterialType,
    },
}

impl RenderMethod {
    pub fn parse(input: &[u8]) -> WResult<Self> {
        let (remaining, raw_flags) = le_u32(input)?;

        Ok((remaining, Self::from_u32(raw_flags)))
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Self::Standard {
                draw_style,
                lighting,
                shading,
                texture_style,
                unknown_bits,
            } => {
                (*draw_style as u32)
                    | ((*lighting as u32) << 2)
                    | ((*shading as u32) << 5)
                    | ((*texture_style as u32) << 7)
                    | ((*unknown_bits as u32) << 11)
            }
            Self::UserDefined { material_type } => (*material_type as u32) | 0x80000000,
        }
    }

    pub fn from_u32(raw_flags: u32) -> Self {
        if raw_flags >> 31 == 1 {
            Self::UserDefined {
                material_type: FromPrimitive::from_u32(raw_flags & !0x80000000).unwrap(),
            }
        } else {
            Self::Standard {
                draw_style: FromPrimitive::from_u32(raw_flags & 0b11).unwrap(),
                lighting: FromPrimitive::from_u32((raw_flags >> 2) & 0b111).unwrap(),
                shading: FromPrimitive::from_u32((raw_flags >> 5) & 0b11).unwrap(),
                texture_style: FromPrimitive::from_u32((raw_flags >> 7) & 0b1111).unwrap(),
                unknown_bits: (raw_flags >> 11) & 0xfffff,
            }
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        match self {
            Self::UserDefined { .. } => self.as_u32().to_le_bytes().to_vec(),
            Self::Standard { .. } => self.as_u32().to_le_bytes().to_vec(),
        }
    }
}

impl From<RenderMethod> for u32 {
    fn from(value: RenderMethod) -> Self {
        value.as_u32()
    }
}

impl std::fmt::Debug for RenderMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard {
                draw_style,
                lighting,
                shading,
                texture_style,
                unknown_bits,
            } => write!(
                f,
                r#"RenderMethod::Standard(0b{:b}) {{
    draw_style: {:?}
    lighting: {:?}
    shading: {:?}
    texture_style: {:?}
    unknown_bits: {:?}
}}"#,
                self.as_u32(),
                draw_style,
                lighting,
                shading,
                texture_style,
                unknown_bits
            ),
            Self::UserDefined { material_type } => write!(
                f,
                r#"RenderMethod::UserDefined(0b{:b}) {{
    material_type: {:?}
}}"#,
                self.as_u32(),
                material_type,
            ),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum DrawStyle {
    Transparent = 0x0,
    Unknown = 0x1,
    Wireframe = 0x2,
    Solid = 0x3,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
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
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum Shading {
    None1 = 0x0,
    None2 = 0x1,
    Gouraud1 = 0x2,
    Gouraud2 = 0x3,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
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
    UnknownTanarus = 0xd,
    Unknown2 = 0xe,
    XXXXX = 0xf,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
/// Source: LanternExtractor
/// (https://github.com/LanternEQ/LanternExtractor/blob/afe174b71ac9f9ab75e259bac2282735b093426d/LanternExtractor/EQ/Wld/DataTypes/MaterialType.cs)
pub enum MaterialType {
    /// Used for boundaries that are not rendered. TextInfoReference can be null or have reference.
    Boundary = 0x0,
    /// Standard diffuse shader
    Diffuse = 0x01,
    /// Diffuse variant
    Diffuse2 = 0x02,
    //// Transparent with 0.5 blend strength
    Transparent50 = 0x05,
    /// Transparent with 0.25 blend strength
    Transparent25 = 0x09,
    /// Transparent with 0.75 blend strength
    Transparent75 = 0x0A,
    /// Non solid surfaces that shouldn't really be masked
    TransparentMaskedPassable = 0x07,
    TransparentAdditiveUnlit = 0x0B,
    TransparentMasked = 0x13,
    Diffuse3 = 0x14,
    Diffuse4 = 0x15,
    TransparentAdditive = 0x17,
    Diffuse5 = 0x19,
    InvisibleUnknown = 0x53,
    Diffuse6 = 0x553,
    CompleteUnknown = 0x1A, // TODO: Analyze this
    Diffuse7 = 0x12,
    Diffuse8 = 0x31,
    InvisibleUnknown2 = 0x4B,
    DiffuseSkydome = 0x0D,     // Need to confirm
    TransparentSkydome = 0x0F, // Need to confirm
    TransparentAdditiveUnlitSkydome = 0x10,
    InvisibleUnknown3 = 0x03,
    CompleteUnknown2 = 0x06, // Found on a "floor" wall in tanarus 'thecity'
}
