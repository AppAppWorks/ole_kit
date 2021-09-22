use std::fs::File;
use std::os::unix::fs::FileExt;
use core::fmt;
use std::fmt::Formatter;
use std::mem::transmute;
use crate::cfb::SectorNumber;

macro_rules! read_type {
    ($self:ident, $offset:expr, $type:ident) => {
        $self.file.read_sized($offset, $type::from_le_bytes)
    }
}

pub struct Signature(pub u64);
crate::impl_for_hex_debug!(Signature, "16");

pub struct VersionNumber(pub u16);
crate::impl_for_hex_debug!(VersionNumber, "4");

pub struct SectorShift(pub u16);
crate::impl_for_hex_debug!(SectorShift, "4");

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SectorCount(pub u32);
crate::impl_for_hex_debug!(SectorCount, "8");

/// The structure at the beginning of a [compound file].
///
/// [compound file]: crate::cfb::Cfb
pub struct Header<'a> {
    file: &'a File,
}

impl<'a> Header<'a> {
    const SIGNATURE: u64 = 8;
    const CLSID: u64 = 16;
    const RESERVED: u64 = 6;

    pub fn new(file: &'a File) -> Self {
        Self { file }
    }

    pub fn is_signature_invalid(&self) -> bool {
        self.signature().0 == 0xe11ab1a1e011cfd0
    }

    /// Identification signature for the [compound file] structure, and MUST be set to the value
    /// 0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1.
    ///
    /// [compound file]: crate::cfb::Cfb
    pub fn signature(&self) -> Signature {
        Signature(read_type!(self, 0, u64))
    }

    /// Version number for nonbreaking changes.
    ///
    /// This field SHOULD be set to 0x003E if the major version field is either 0x0003 or 0x0004.
    pub fn minor_version(&self) -> VersionNumber {
        VersionNumber(read_type!(self, Self::SIGNATURE + Self::CLSID, u16))
    }

    /// Version number for breaking changes.
    ///
    /// This field MUST be set to either 0x0003 (version 3) or 0x0004 (version 4).
    pub fn major_version(&self) -> VersionNumber {
        VersionNumber(read_type!(self, Self::SIGNATURE + Self::CLSID + 2, u16))
    }

    /// This field MUST be set to 0xFFFE.
    ///
    /// This field is a byte order mark for all integer fields, specifying little-endian byte order.
    pub fn byte_order(&self) -> u16 {
        read_type!(self, Self::SIGNATURE + Self::CLSID + 4, u16)
    }

    /**
    This field MUST be set to 0x0009, or 0x000c, depending on the Major Version field.

    This field specifies the sector size of the compound file as a power of 2.

    - If Major Version is 3, the Sector Shift MUST be 0x0009, specifying a sector size of 512 bytes.
    - If Major Version is 4, the Sector Shift MUST be 0x000C, specifying a sector size of 4096 bytes.
     */
    pub fn sector_shift(&self) -> SectorShift {
        SectorShift(read_type!(self, Self::SIGNATURE + Self::CLSID + 6, u16))
    }

    /**
    This field MUST be set to 0x0006.

    This field specifies the sector size of the Mini Stream as a power of 2. The sector size of the
    Mini Stream MUST be 64 bytes.
     */
    pub fn mini_sector_shift(&self) -> SectorShift {
        SectorShift(read_type!(self, Self::SIGNATURE + Self::CLSID + 8, u16))
    }

    /// This integer field contains the count of the number of [directory] sectors in the
    /// [compound file].
    ///
    /// - If Major Version is 3, the Number of Directory Sectors MUST be zero. This field is not
    /// supported for version 3 compound files.
    ///
    /// [directory]: crate::cfb::directory::Directory
    /// [compound file]: crate::cfb::Cfb
    pub fn no_of_directory_sectors(&self) -> Option<SectorCount> {
        let raw = read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED, u32);
        if raw == 0 { None } else { Some(SectorCount(raw)) }
    }

    /// This integer field contains the count of the number of [FAT] sectors in the [compound file].
    ///
    /// [FAT]: crate::cfb::fat::Fat
    /// [compound file]: crate::cfb::Cfb
    pub fn no_of_fat_sectors(&self) -> SectorCount {
        SectorCount(read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 4, u32))
    }

    /// This integer field contains the starting sector number for the [directory] stream.
    ///
    /// [directory]: crate::cfb::directory::Directory
    pub fn first_directory_sector_location(&self) -> SectorNumber {
        SectorNumber(read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 8, u32))
    }

    /// This integer field MAY contain a sequence number that is incremented every time the compound
    /// file is saved by an implementation that supports file transactions. This is the field that
    /// MUST be set to all zeroes if file transactions are not implemented.
    pub fn transaction_signature_number(&self) -> u32 {
        read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 12, u32)
    }

    /// This integer field MUST be set to 0x00001000.
    ///
    /// This field specifies the maximum size of a user-defined data stream that is allocated from
    /// the mini FAT and mini stream, and that cutoff is 4,096 bytes. Any user-defined data stream
    /// that is greater than or equal to this cutoff size must be allocated as normal sectors from
    /// the FAT.
    pub fn mini_stream_cutoff_size(&self) -> u32 {
        read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 16, u32)
    }

    /// This integer field contains the starting sector number for the mini FAT.
    pub fn first_mini_fat_sector_location(&self) -> SectorNumber {
        SectorNumber(read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 20, u32))
    }

    /// This integer field contains the count of the number of mini FAT sectors in the compound file.
    pub fn no_of_mini_fat_sectors(&self) -> SectorCount {
        SectorCount(read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 24, u32))
    }

    /// This integer field contains the starting sector number for the [DIFAT].
    ///
    /// [DIFAT]: self::Difat
    pub fn first_difat_sector_location(&self) -> SectorNumber {
        SectorNumber(read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 28, u32))
    }

    /// This integer field contains the count of the number of [DIFAT] sectors in the [compound file].
    ///
    /// [DIFAT]: self::Difat
    /// [compound file]: crate::cfb::Cfb
    pub fn no_of_difat_sectors(&self) -> SectorCount {
        SectorCount(read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 32, u32))
    }

    /**
    This array of 32-bit integer fields contains the first 109 [FAT] sector locations of the
    compound file.

    - For version 4 compound files, the header size (512 bytes) is less than the sector size (4,096
    bytes), so the remaining part of the header (3,584 bytes) MUST be filled with all zeroes.

    [FAT]: crate::cfb::fat::Fat
     */
    pub(crate) fn difat(&self) -> Difat<109> {
        let mut bytes = [0u8; 109 * std::mem::size_of::<u32>()];
        self.file.read_at(&mut bytes, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 36);
        Difat(unsafe { transmute(bytes) })
    }

    /// Returns the sector number of a [FAT] sector by its index in the [DIFAT].
    ///
    /// [FAT]: crate::cfb::fat::Fat
    /// [DIFAT]: self::Difat
    pub(crate) fn sector_no_of_fat(&self, index: SectorNumber) -> SectorNumber {
        SectorNumber(read_type!(self, Self::SIGNATURE + Self::CLSID + 10 + Self::RESERVED + 36 + index.byte_offset(std::mem::size_of::<u32>()), u32))
    }

    // TODO: map also the DIFAT sectors outside of the header
}

/// `double-indirect file allocation table`
///
/// A structure that is used to locate [FAT] sectors in a [compound file].
///
/// [FAT]: crate::cfb::fat::Fat
/// [compound file]: crate::cfb::Cfb
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Difat<const N: usize>([SectorNumber; N]);

impl<'a> fmt::Debug for Header<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_map();
        crate::debug_map_method_reflection!(fmt, self,
            signature,
            minor_version,
            major_version,
            byte_order,
            sector_shift,
            mini_sector_shift,
            no_of_directory_sectors,
            no_of_fat_sectors,
            first_directory_sector_location,
            transaction_signature_number,
            mini_stream_cutoff_size,
            first_mini_fat_sector_location,
            no_of_mini_fat_sectors,
            first_difat_sector_location,
            no_of_difat_sectors,
            difat
        );
        fmt.finish()
    }
}

pub(crate) trait FileSlice {
    /// Reads bytes from the file by the offset and size.
    fn read_bytes(&self, offset: u64, size: usize) -> Vec<u8>;

    /// Reads a value of a fix-sized data type from the file by the offset and constructor.
    fn read_sized<T, const N: usize>(&self, offset: u64, constructor: impl FnOnce([u8; N]) -> T) -> T;
}

impl FileSlice for File {
    fn read_bytes(&self, offset: u64, size: usize) -> Vec<u8> {
        let mut bytes = vec![0; size];
        self.read_at(&mut bytes, offset);
        bytes
    }

    fn read_sized<T, const N: usize>(&self, offset: u64, constructor: impl FnOnce([u8; N]) -> T) -> T {
        let mut bytes = [0; N];
        self.read_at(&mut bytes, offset);
        constructor(bytes)
    }
}
