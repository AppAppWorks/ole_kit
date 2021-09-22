use std::ops::{Add, Div, Rem};
use core::fmt;
use std::fmt::Formatter;
use crate::cfb::header::SectorCount;
use std::convert::TryInto;

/** The sector number can be used as an index into the [FAT] array to continue along the chain.

Special values are reserved for chain terminators (ENDOFCHAIN = 0xFFFFFFFE), free sectors
(FREESECT = 0xFFFFFFFF), and sectors that contain storage for FAT sectors (FATSECT = 0xFFFFFFFD) or
DIFAT Sectors (DIFSECT = 0xFFFFFFC), which are not chained in the same way as the others.

The locations of FAT sectors are read from the DIFAT. The FAT is represented in itself, but not
by a chain. A special reserved sector number (FATSECT = 0xFFFFFFFD) is used to mark sectors that are
allocated to the FAT.

A sector number can be converted into a byte offset into the file by using the following
formula: (sector number + 1) x `Sector Size`. This implies that sector #0 of the file begins at
byte offset `Sector Size`, not at 0.

[FAT]: crate::cfb::fat::Fat
 */
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SectorNumber(pub u32);

impl SectorNumber {
    /// 0xFFFFFFC
    ///
    /// DIFAT Sectors (DIFSECT = 0xFFFFFFC), which are not chained in the same way as the others.
    pub const DIFSECT: Self = Self(0xFFFFFFC);
    /// 0xFFFFFFFD
    ///
    /// Sectors that contain storage for FAT sectors (FATSECT = 0xFFFFFFFD).
    pub const FATSECT: Self = Self(0xFFFFFFFD);
    /// 0xFFFFFFFE
    ///
    /// Chain terminators (ENDOFCHAIN = 0xFFFFFFFE).
    pub const ENDOFCHAIN: Self = Self(0xFFFFFFFE);
    /// 0xFFFFFFFF
    ///
    /// Free sectors (FREESECT = 0xFFFFFFFF).
    pub const FREESECT: Self = Self(0xFFFFFFFF);

    pub fn is_difat(&self) -> bool {
        self == &Self::DIFSECT
    }

    pub fn is_fat(&self) -> bool {
        self == &Self::FATSECT
    }

    pub fn is_end_of_chain(&self) -> bool {
        self == &Self::ENDOFCHAIN
    }

    pub fn is_free(&self) -> bool {
        self == &Self::FREESECT
    }

    pub fn is_other(&self) -> bool {
        !(self.is_difat() || self.is_end_of_chain() || self.is_fat() || self.is_free())
    }
}

impl SectorNumber {
    pub(crate) fn byte_offset(&self, sector_size: impl TryInto<u64>) -> u64 {
        self.0 as u64 * sector_size.try_into().unwrap_or(0)
    }
}

impl Add<u32> for SectorNumber {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Div<SectorCount> for SectorNumber {
    type Output = Self;

    fn div(self, rhs: SectorCount) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Rem<SectorCount> for SectorNumber {
    type Output = Self;

    fn rem(self, rhs: SectorCount) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl fmt::Debug for SectorNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            Self::FREESECT => "FREESECT".to_string(),
            Self::FATSECT => "FATSECT".to_string(),
            Self::ENDOFCHAIN => "ENDOFCHAIN".to_string(),
            Self::DIFSECT => "DIFSECT".to_string(),
            Self(v) => format!("0x{:08X}", v),
        };
        f.write_str(&format!("SectorNumber({})", str))
    }
}