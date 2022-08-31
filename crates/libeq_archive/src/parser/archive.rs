use std::collections::BTreeMap;

use nom::bytes::complete::take;
use nom::multi::{count, fold_many0};
use nom::number::complete::le_u32;
use nom::IResult;

use super::{Block, Footer, Header, IndexEntry};

///
/// ---------------------
/// |                   |
/// |      Header       |
/// |                   |
/// ---------------------
/// |                   |
/// |     File Data     |
/// |                   |
/// ---------------------
/// |                   |
/// |      Index        |
/// |                   |
/// ---------------------
/// |                   |
/// |      Footer       |
/// |                   |
/// ---------------------
///

#[derive(Debug)]
pub struct Archive {
    pub header: Header,
    pub blocks: BTreeMap<usize, Block>,
    pub index_entries: Vec<IndexEntry>,
    /// The footer does not seem to be present in all files (global_chr1.s3d for example).
    pub footer: Option<Footer>,
}

impl Archive {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, header) = Header::parse(input)?;
        let (i, all_block_data) = take(header.index_offset - Header::SIZE as u32)(i)?;
        let (i, index_entry_count) = le_u32(i)?;
        let (i, index_entries) = count(IndexEntry::parse, index_entry_count as usize)(i)?;

        let (i, footer) = if i.len() > 0 {
            Footer::parse(i).map(|(i, f)| (i, Some(f)))?
        } else {
            (i, None)
        };

        let (_, (_, blocks)) = fold_many0(
            Block::parse,
            // (current offset into the file, the block tree we're building)
            || (Header::SIZE, BTreeMap::new()),
            |(offset, mut blocks), block| {
                let next_offset = offset + block.size();
                blocks.insert(offset, block);
                (next_offset, blocks)
            },
        )(all_block_data)?;

        Ok((
            i,
            Archive {
                header,
                blocks,
                index_entries,
                footer,
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let index_entry_count_bytes = (self.index_entries.len() as u32).to_le_bytes();
        let index_entry_bytes: Vec<_> = self
            .index_entries
            .iter()
            .flat_map(|e| e.to_bytes())
            .collect();
        let block_bytes: Vec<_> = self.blocks.values().flat_map(|b| b.to_bytes()).collect();
        [
            &self.header.to_bytes()[..],
            &block_bytes,
            &index_entry_count_bytes,
            &index_entry_bytes,
            &self.footer.as_ref().map_or(vec![], |f| f.to_bytes())[..],
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Read;

    #[test]
    fn it_parses() {
        let mut fixture = File::open("fixtures/gfaydark.s3d").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, archive) = Archive::parse(&fixture_data).unwrap();

        assert_eq!(
            archive.header,
            Header {
                index_offset: 0x219dbf,
                magic_number: u32::from_le_bytes(*b"PFS "),
                version: 0x00020000,
            },
        );
        assert_eq!(
            archive.blocks.values().map(|b| b.size()).sum::<usize>(),
            (archive.header.index_offset - Header::SIZE as u32) as usize
        );
        assert_eq!(archive.index_entries.len(), 41);
        assert_eq!(
            archive.footer,
            Some(Footer {
                footer_string: b"STEVE".to_vec(),
                timestamp: 0x5b28ad36,
            }),
        );
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark.s3d").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, archive) = Archive::parse(&fixture_data).unwrap();

        assert_eq!(archive.to_bytes(), fixture_data);
    }
}
