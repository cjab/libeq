//! Raw, borrowed binary EQGZ version 1 and 2 zone descriptors.
//!
//! Transforms, extension words, regions, and lights are preserved without
//! assigning coordinate conventions or interpreting their opaque fields.

/// A failure to decode a supported binary zone descriptor.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("invalid EQGZ magic")]
    InvalidMagic,
    #[error("unsupported EQGZ version {version}")]
    UnsupportedVersion { version: u32 },
    #[error("truncated {context} at byte {offset}: need {needed} bytes, have {remaining}")]
    Truncated {
        context: &'static str,
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    #[error("byte count overflow in {context}")]
    CountOverflow { context: &'static str },
    #[error("invalid or unterminated string reference {offset}")]
    InvalidStringReference { offset: u32 },
    #[error("placement model index {index} is outside model count {model_count}")]
    InvalidModelReference { index: u32, model_count: usize },
}

/// A raw zone descriptor. String offsets address `string_table` bytes.
#[derive(Debug)]
pub struct Zone<'a> {
    pub version: u32,
    pub string_table: &'a [u8],
    /// String offsets, with the all-ones sentinel represented by `None`.
    pub models: Vec<Option<u32>>,
    pub placements: Vec<Placement<'a>>,
    pub regions: Vec<Region>,
    pub lights: Vec<Light>,
    /// All bytes following the counted records; retained, not interpreted.
    pub trailing_data: &'a [u8],
}

impl Zone<'_> {
    /// Resolve a NUL-terminated byte string, excluding its terminator.
    /// Non-UTF-8 bytes and offsets into the middle of a string are permitted.
    pub fn string(&self, offset: u32) -> Result<&[u8], ParseError> {
        let start = usize::try_from(offset).ok();
        let bytes = start
            .and_then(|start| self.string_table.get(start..))
            .ok_or(ParseError::InvalidStringReference { offset })?;
        let end = bytes
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(ParseError::InvalidStringReference { offset })?;
        Ok(&bytes[..end])
    }
}

/// Raw placement values; floating-point bit patterns are preserved.
#[derive(Debug)]
pub struct Placement<'a> {
    pub model_index: Option<u32>,
    pub name_offset: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: f32,
    /// Counted version 2 words, or an empty slice for version 1.
    pub extension_data: &'a [u8],
}
impl Placement<'_> {
    pub fn extension_words(&self) -> impl Iterator<Item = u32> + '_ {
        self.extension_data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

#[derive(Debug)]
pub struct Region {
    pub name_offset: u32,
    pub data: [u32; 9],
}
#[derive(Debug)]
pub struct Light {
    pub name_offset: u32,
    pub data: [u32; 7],
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn require(&self, needed: usize, context: &'static str) -> Result<(), ParseError> {
        let remaining = self.bytes.len() - self.offset;
        if needed > remaining {
            return Err(ParseError::Truncated {
                context,
                offset: self.offset,
                needed,
                remaining,
            });
        }
        Ok(())
    }
    fn take(&mut self, count: usize, context: &'static str) -> Result<&'a [u8], ParseError> {
        self.require(count, context)?;
        let start = self.offset;
        self.offset += count;
        Ok(&self.bytes[start..self.offset])
    }
    fn word(&mut self, context: &'static str) -> Result<u32, ParseError> {
        let bytes = self.take(4, context)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    fn words<const N: usize>(&mut self, context: &'static str) -> Result<[u32; N], ParseError> {
        let mut words = [0; N];
        for word in &mut words {
            *word = self.word(context)?;
        }
        Ok(words)
    }
}
fn byte_count(count: u32, stride: usize, context: &'static str) -> Result<usize, ParseError> {
    usize::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(stride))
        .ok_or(ParseError::CountOverflow { context })
}
fn nullable(value: u32) -> Option<u32> {
    (value != u32::MAX).then_some(value)
}

/// Decode a complete counted descriptor, retaining any trailing bytes.
///
/// Every referenced string must terminate within the string table. Placement
/// model indices must address the model table, but may address a nullable entry.
/// Count-based minimum sizes are checked before allocating record vectors.
/// Unknown versions are rejected rather than parsed using another layout.
pub fn parse(input: &[u8]) -> Result<Zone<'_>, ParseError> {
    let mut reader = Reader {
        bytes: input,
        offset: 0,
    };
    if reader.take(4, "magic")? != b"EQGZ" {
        return Err(ParseError::InvalidMagic);
    }
    let version = reader.word("version")?;
    if !matches!(version, 1 | 2) {
        return Err(ParseError::UnsupportedVersion { version });
    }
    let [strings, models, placements, regions, lights] = reader.words("header")?;
    let mut minimum = 0usize;
    for (count, stride) in [
        (strings, 1),
        (models, 4),
        (placements, if version == 1 { 36 } else { 40 }),
        (regions, 40),
        (lights, 32),
    ] {
        minimum = minimum
            .checked_add(byte_count(count, stride, "record counts")?)
            .ok_or(ParseError::CountOverflow {
                context: "record counts",
            })?;
    }
    reader.require(minimum, "counted records")?;
    let string_table = reader.take(byte_count(strings, 1, "string table")?, "string table")?;
    // Any offset at or before the final NUL has a terminator. This validates
    // repeated references in constant time after one scan of the string table.
    let last_nul = string_table.iter().rposition(|&byte| byte == 0);
    let check_string = |offset: u32| -> Result<(), ParseError> {
        if usize::try_from(offset)
            .ok()
            .zip(last_nul)
            .is_some_and(|(offset, last)| offset <= last)
        {
            Ok(())
        } else {
            Err(ParseError::InvalidStringReference { offset })
        }
    };
    let mut zone = Zone {
        version,
        string_table,
        models: Vec::new(),
        placements: Vec::new(),
        regions: Vec::new(),
        lights: Vec::new(),
        trailing_data: &[],
    };
    for _ in 0..models {
        let offset = nullable(reader.word("model")?);
        if let Some(offset) = offset {
            check_string(offset)?;
        }
        zone.models.push(offset);
    }
    for _ in 0..placements {
        let model_index = nullable(reader.word("placement model")?);
        if let Some(index) = model_index.filter(|&index| index >= models) {
            return Err(ParseError::InvalidModelReference {
                index,
                model_count: zone.models.len(),
            });
        }
        let name_offset = reader.word("placement name")?;
        check_string(name_offset)?;
        let position = reader.words("placement position")?.map(f32::from_bits);
        let rotation = reader.words("placement rotation")?.map(f32::from_bits);
        let scale = f32::from_bits(reader.word("placement scale")?);
        let extension_data = if version == 2 {
            let count = reader.word("placement extension count")?;
            reader.take(
                byte_count(count, 4, "placement extension")?,
                "placement extension",
            )?
        } else {
            &[]
        };
        zone.placements.push(Placement {
            model_index,
            name_offset,
            position,
            rotation,
            scale,
            extension_data,
        });
    }
    for _ in 0..regions {
        let name_offset = reader.word("region name")?;
        check_string(name_offset)?;
        zone.regions.push(Region {
            name_offset,
            data: reader.words("region data")?,
        });
    }
    for _ in 0..lights {
        let name_offset = reader.word("light name")?;
        check_string(name_offset)?;
        zone.lights.push(Light {
            name_offset,
            data: reader.words("light data")?,
        });
    }
    zone.trailing_data = &input[reader.offset..];
    Ok(zone)
}
