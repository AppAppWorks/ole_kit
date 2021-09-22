use crate::cfb::directory::entry::CommonProps;
use std::fs::File;
use crate::cfb::directory::entry::metadata::{ColorFlag, StreamSize};
use crate::cfb::directory::StreamID;
use crate::cfb::header::FileSlice;
use std::convert::TryInto;
use std::os::unix::fs::FileExt;
use chrono::{NaiveDateTime, Duration};
use crate::cfb::fat::sector_number::SectorNumber;
use crate::cfb::Cfb;
use crate::cfb::fat;
use crate::cfb::SectorCount;
use core::fmt;
use std::fmt::Formatter;

macro_rules! impl_cls_id {
    ($type:ident) => {
        impl<'a> $type<'a> {
            /// This field contains an object class GUID, if this entry is for a storage object or root
            /// storage object.
            ///
            /// For a stream object, this field MUST be set to all zeroes. A value containing all zeroes in
            /// a storage or root storage directory entry is valid, and indicates that no object class is
            /// associated with the storage. If an implementation of the file format enables applications to
            /// create storage objects without explicitly setting an object class GUID, it MUST write all
            /// zeroes by default. If this value is not all zeroes, the object class GUID can be used as a
            /// parameter to start applications.
            pub fn cls_id(&self) -> [u8; 16] {
                let mut bytes = [0; Self::CLSID as usize];
                self.file.read_at(&mut bytes, self.offset() + Self::NAME + 16);
                bytes
            }
        }
    };
}

macro_rules! impl_state_bits {
    ($type:ident) => {
        impl<'a> $type<'a> {
            /// This field contains the user-defined flags if this entry is for a storage object or root
            /// storage object.
            ///
            /// For a stream object, this field SHOULD be set to all zeroes because many implementations
            /// provide no way for applications to retrieve state bits from a stream object. If an
            /// implementation of the file format enables applications to create storage objects without
            /// explicitly setting state bits, it MUST write all zeroes by default.
            pub fn state_bits(&self) -> u32 {
                self.read_sized(Self::NAME + 16 + Self::CLSID, u32::from_le_bytes)
            }
        }
    };
}

macro_rules! impl_starting_sector_location {
    ($type:ident) => {
        impl<'a> $type<'a> {
            /// This field contains the first sector location if this is a stream object.
            ///
            /// For a root storage object, this field MUST contain the first sector of the mini stream, if
            /// the mini stream exists. For a storage object, this field MUST be set to all zeroes.
            pub fn starting_sector_location(&self) -> SectorNumber {
                SectorNumber(self.read_sized(Self::NAME + 16 + Self::CLSID + Self::STATE_BITS + Self::TIME + Self::TIME, u32::from_le_bytes))
            }
        }
    };
}

macro_rules! impl_stream_size {
    ($type:ident) => {
        impl<'a> $type<'a> {
            /**
            This 64-bit integer field contains the size of the user-defined data if this is a stream object.

            For a root storage object, this field contains the size of the mini stream. For a storage
            object, this field MUST be set to all zeroes.

            - For a version 3 compound file 512-byte sector size, the value of this field MUST be less than
            or equal to 0x80000000. (Equivalently, this requirement can be stated: the size of a stream or
            of the mini stream in a version 3 compound file MUST be less than or equal to 2 gigabytes (GB).)
            Note that as a consequence of this requirement, the most significant 32 bits of this field MUST
            be zero in a version 3 compound file. However, implementers should be aware that some older
            implementations did not initialize the most significant 32 bits of this field, and these bits
            might therefore be nonzero in files that are otherwise valid version 3 compound files. Although
            this document does not normatively specify parser behavior, it is recommended that parsers
            ignore the most significant 32 bits of this field in version 3 compound files, treating it as if
            its value were zero, unless there is a specific reason to do otherwise (for example, a parser
            whose purpose is to verify the correctness of a compound file).
             */
            pub fn stream_size(&self) -> StreamSize {
                StreamSize(self.read_sized(Self::NAME + 16 + Self::CLSID + Self::STATE_BITS + Self::TIME + Self::TIME + 4,
                                u64::from_le_bytes))
            }
        }
    };
}

/// A storage object in a compound file that must be accessed before any other storage objects
/// and stream objects are referenced. It is the uppermost parent object in the storage object
/// and stream object hierarchy.
pub struct RootStorage<'a> {
    offset: u64,
    file: &'a File,
}

impl_cls_id!(RootStorage);
impl_state_bits!(RootStorage);
impl_starting_sector_location!(RootStorage);
impl_stream_size!(RootStorage);

impl<'a> RootStorage<'a> {
    pub fn mini_stream_range(&self) -> (SectorNumber, StreamSize) {
        (self.starting_sector_location(), self.stream_size())
    }

    pub fn mini_stream_bytes(&self, cfb: &Cfb) -> Vec<u8> {
        let mut fat_cache = fat::cache::Cache::new(cfb);

        let no_of_sectors_per_fat = SectorCount(cfb.sector_size >> 2);

        let mut idx = self.starting_sector_location();

        let mut stream_bytes = Vec::new();
        stream_bytes.reserve_exact(self.stream_size().0 as usize);

        loop {
            let mut sector_bytes = cfb.sector_bytes(idx);

            stream_bytes.append(&mut sector_bytes);

            let fat = fat_cache.fat(idx);

            let next = fat.sector_number((idx % no_of_sectors_per_fat).0);

            if !next.is_other() { break }

            idx = next;
        }

        stream_bytes
    }
}

/// An object in a compound file that is analogous to a file system directory. The parent object
/// of a storage object must be another storage object or the root storage object.
pub struct Storage<'a> {
    offset: u64,
    file: &'a File,
}

impl_cls_id!(Storage);
impl_state_bits!(Storage);

impl<'a> Storage<'a> {
    /// This field contains the creation time for a storage object, or all zeroes to indicate that
    /// the creation time of the storage object was not recorded.
    ///
    /// The Windows FILETIME structure is used to represent this field in UTC. For a stream object,
    /// this field MUST be all zeroes. For a root storage object, this field MUST be all zeroes, and
    /// the creation time is retrieved or set on the compound file itself.
    pub fn creation_time(&self) -> Option<NaiveDateTime> {
        let nano_secs = self.read_sized(Self::NAME + 16 + Self::CLSID + Self::STATE_BITS, u64::from_le_bytes);
        if nano_secs != 0 {
            let date_time =
                NaiveDateTime::from_timestamp((nano_secs / 1_000_000_000) as i64,
                                              (nano_secs % 1_000_000_000) as u32);
            Some(date_time - Duration::seconds(11_644_473_600))
        } else {
            None
        }
    }

    /// This field contains the modification time for a storage object, or all zeroes to indicate
    /// that the modified time of the storage object was not recorded.
    ///
    /// The Windows FILETIME structure is used to represent this field in UTC. For a stream object,
    /// this field MUST be all zeroes. For a root storage object, this field MAY be set to all
    /// zeroes, and the modified time is retrieved or set on the compound file itself.
    pub fn modified_time(&self) -> Option<NaiveDateTime> {
        let nano_secs = self.read_sized(Self::NAME + 16 + Self::CLSID + Self::STATE_BITS + Self::TIME, u64::from_le_bytes);
        if nano_secs != 0 {
            let date_time =
                NaiveDateTime::from_timestamp((nano_secs / 1_000_000_000) as i64,
                                              (nano_secs % 1_000_000_000) as u32);
            Some(date_time - Duration::seconds(11_644_473_600))
        } else {
            None
        }
    }
}

/// An object in a compound file that is analogous to a file system file. The parent object of a
/// stream object must be a storage object or the root storage object.
pub struct Stream<'a> {
    offset: u64,
    file: &'a File,
}

impl_starting_sector_location!(Stream);
impl_stream_size!(Stream);

impl<'a> Stream<'a> {
    pub fn stream_bytes(&self, cfb: &Cfb, root_entry_bytes: Option<Vec<u8>>) -> Vec<u8> {
        let mut fat_cache = fat::cache::Cache::new(cfb);

        let no_of_sectors_per_fat = SectorCount(cfb.sector_size >> 2);

        let mut idx = self.starting_sector_location();

        let mut stream_bytes = Vec::new();
        stream_bytes.reserve_exact(self.stream_size().0 as usize);

        if cfb.header().mini_stream_cutoff_size() as u64 <= self.stream_size().0 {
            loop {
                let mut sector_bytes = cfb.sector_bytes(idx);

                stream_bytes.append(&mut sector_bytes);

                let fat = fat_cache.fat(idx);

                let next = fat.sector_number((idx % no_of_sectors_per_fat).0);

                if !next.is_other() { break }

                idx = next;
            }
        } else {
            let root_entry_bytes = root_entry_bytes.unwrap_or_else(||
                cfb.mini_stream_bytes());

            loop {
                let mini_sector = &root_entry_bytes[idx.0 as usize * 64..][..64];

                stream_bytes.clone_from_slice(mini_sector);

                let mini_fat = fat_cache.mini_fat(idx);

                let next = mini_fat.sector_number((idx % no_of_sectors_per_fat).0);

                if !next.is_other() { break }

                idx = next;
            }
        }

        stream_bytes
    }
}

macro_rules! impl_entry_props {
    ($type:ident) => {
        impl<'a> CommonProps<'a> for $type<'a> {
            fn new(offset: u64, file: &'a File) -> Result<Self, String> {
                Ok(Self { offset, file })
            }

            fn offset(&self) -> u64 {
                self.offset
            }

            fn file(&self) -> &File {
                self.file
            }

            fn name(&self) -> String {
                let bytes = self.read_bytes(0, self.name_length() as usize - 1);
                let utf16 = unsafe {
                    std::slice::from_raw_parts(bytes.as_ptr() as *const u16, bytes.len() >> 1)
                };
                String::from_utf16_lossy(&utf16)
            }

            fn name_length(&self) -> u16 {
                self.read_sized(Self::NAME, u16::from_le_bytes)
            }

            fn color_flag(&self) -> ColorFlag {
                let byte = self.read_sized(Self::NAME + 3, u8::from_ne_bytes);
                byte.try_into().unwrap()
            }

            fn left_sibling_id(&self) -> Option<StreamID> {
                let raw_value = self.read_sized(Self::NAME + 4, u32::from_le_bytes);
                if raw_value != u32::MAX { Some(StreamID(raw_value)) } else { None }
            }

            fn right_sibling_id(&self) -> Option<StreamID> {
                let raw_value = self.read_sized(Self::NAME + 8, u32::from_le_bytes);
                if raw_value != u32::MAX { Some(StreamID(raw_value)) } else { None }
            }

            fn child_id(&self) -> Option<StreamID> {
                let raw_value = self.read_sized(Self::NAME + 12, u32::from_le_bytes);
                if raw_value != u32::MAX { Some(StreamID(raw_value)) } else { None }
            }
        }

        impl<'a> FileSlice for $type<'a> {
            #[inline]
            fn read_bytes(&self, offset: u64, size: usize) -> Vec<u8> {
                self.file.read_bytes(offset + self.offset, size)
            }

            #[inline]
            fn read_sized<T, const N: usize>(&self, offset: u64, constructor: impl FnOnce([u8; N]) -> T) -> T {
                self.file.read_sized(offset + self.offset, constructor)
            }
        }
    };
}

impl_entry_props!(RootStorage);
impl_entry_props!(Storage);
impl_entry_props!(Stream);

impl<'a> fmt::Debug for RootStorage<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_map();
        crate::debug_map_method_reflection!(
            fmt,
            self,
            name,
            name_length,
            color_flag,
            left_sibling_id,
            right_sibling_id,
            child_id,
            cls_id,
            state_bits,
            starting_sector_location,
            mini_stream_range
        );
        fmt.finish()
    }
}

impl<'a> fmt::Debug for Storage<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_map();
        crate::debug_map_method_reflection!(
            fmt,
            self,
            name,
            name_length,
            color_flag,
            left_sibling_id,
            right_sibling_id,
            child_id,
            cls_id,
            state_bits,
            creation_time,
            modified_time
        );
        fmt.finish()
    }
}

impl<'a> fmt::Debug for Stream<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_map();
        crate::debug_map_method_reflection!(
            fmt,
            self,
            name,
            name_length,
            color_flag,
            left_sibling_id,
            right_sibling_id,
            child_id,
            starting_sector_location,
            stream_size
        );
        fmt.finish()
    }
}