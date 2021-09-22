use crate::cfb::fat::sector_number::SectorNumber;
use std::convert::TryInto;
use core::fmt;
use std::fmt::Formatter;

pub mod sector_number;
pub(crate) mod cache;

/// The FAT is an array of [sector numbers] that represent the allocation of space within the file,
/// grouped into FAT sectors. Each stream is represented in the FAT by a sector chain, in much the
/// same fashion as a FAT file system.
/// 
/// [sector numbers]: crate::cfb::fat::sector_number::SectorNumber
pub(crate) struct Fat {
    pub(crate) data: Vec<u8>,
}

impl Fat {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    const U32_SIZE: usize = std::mem::size_of::<u32>();

    pub(crate) fn sector_numbers(&self) -> Vec<SectorNumber> {
        (0..(self.data.len() / Self::U32_SIZE))
            .filter_map(|i| self.data[i * Self::U32_SIZE..][..Self::U32_SIZE].try_into().ok())
            .map(|slice|
                SectorNumber(u32::from_le_bytes(slice))
            )
            .collect()
    }

    pub(crate) fn sector_number(&self, index: u32) -> SectorNumber {
        let data = self.data[(index as usize * Self::U32_SIZE)..][..Self::U32_SIZE].to_owned();
        SectorNumber(u32::from_le_bytes(data.try_into().unwrap()))
    }
}

impl fmt::Debug for Fat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.sector_numbers())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfb::Cfb;

    #[test]
    fn basic_read_write() {
        let a = [10; 12];
        let fat = Fat { data: a.to_vec() };
        assert_eq!(fat.sector_numbers(), [SectorNumber(168430090), SectorNumber(168430090), SectorNumber(168430090)]);
        assert_eq!(fat.sector_number(2), SectorNumber(168430090));
    }

    #[test]
    fn doc_fat_sector() {
        let expected = [SectorNumber(0x00000001), SectorNumber(0x00000002),
            SectorNumber(0x00000003), SectorNumber(0x00000006), SectorNumber(0x00000005),
            SectorNumber(0x0000000A), SectorNumber(0x00000007), SectorNumber(0x00000008),
            SectorNumber(0x00000009), SectorNumber::ENDOFCHAIN, SectorNumber(0x0000000B),
            SectorNumber(0x0000000C), SectorNumber(0x0000000D), SectorNumber(0x0000000E),
            SectorNumber(0x0000000F), SectorNumber::ENDOFCHAIN, SectorNumber(0x00000011),
            SectorNumber(0x00000012), SectorNumber(0x00000013), SectorNumber(0x00000014),
            SectorNumber(0x00000015), SectorNumber(0x00000016), SectorNumber(0x00000017),
            SectorNumber::ENDOFCHAIN, SectorNumber(0x0000001A), SectorNumber::ENDOFCHAIN,
            SectorNumber::ENDOFCHAIN, SectorNumber::ENDOFCHAIN, SectorNumber::FATSECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT,
            SectorNumber::FREESECT, SectorNumber::FREESECT, SectorNumber::FREESECT];

        let cfb = Cfb::from_path("tests_rsc/testing.doc").unwrap();
        let fat = cfb.fat_by_stream_sector_no(SectorNumber(0));

        assert_eq!(fat.sector_numbers(), expected);
    }
}
