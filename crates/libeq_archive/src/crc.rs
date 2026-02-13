use crate::parser::IndexEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FilenameCrc(u32);

impl FilenameCrc {
    pub const DIRECTORY: FilenameCrc = FilenameCrc(IndexEntry::DIRECTORY_CRC);

    pub fn new(filename: &str) -> Self {
        filename
            .bytes()
            .chain(vec![0u8]) // Add null string terminator back in
            .fold(0, |crc, byte| {
                let idx = ((crc >> 24) ^ (byte as u32)) & 0xff;
                (crc << 8) ^ CRC_TABLE[idx as usize]
            })
            .into()
    }
}

impl From<&str> for FilenameCrc {
    fn from(filename: &str) -> Self {
        FilenameCrc::new(filename)
    }
}

impl From<u32> for FilenameCrc {
    fn from(raw: u32) -> Self {
        FilenameCrc(raw)
    }
}

const CRC_TABLE: [u32; 256] = build_crc_table();
const fn build_crc_table() -> [u32; 256] {
    const TABLE_SIZE: usize = 256;
    let mut crc_table: [u32; 256] = [0; TABLE_SIZE];

    let mut idx = 0;
    while idx < TABLE_SIZE {
        let mut crc: u32 = (idx as u32) << 24;

        let mut round = 0;
        while round < 8 {
            crc = if (crc & 0x80000000) != 0 {
                (crc << 1) ^ 0x04c11db7
            } else {
                crc << 1
            };
            round += 1;
        }

        crc_table[idx] = crc;
        idx += 1;
    }
    crc_table
}
