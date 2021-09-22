mod metadata;
mod impls;

use std::fs::File;
use crate::cfb::header::{FileSlice};
use crate::cfb::directory::entry::metadata::{ObjectType, ColorFlag};
use std::convert::TryInto;
use crate::cfb::directory::StreamID;
use crate::cfb::directory::entry::impls::{RootStorage, Storage, Stream};

/**
The directory entry array is an array of directory entries that are grouped into a directory sector.
Each storage object or stream object within a compound file is represented by a single directory
entry. The space for the directory sectors that are holding the array is allocated from the FAT.

The valid values for a stream ID, which are used in the [Child ID],
[Right Sibling ID], and [Left Sibling ID] fields, are 0 through MAXREGSID
(0xFFFFFFFA). The special value NOSTREAM (0xFFFFFFFF) is used as a terminator.

The directory entry size is fixed at 128 bytes. The name in the directory entry is limited to 32
Unicode UTF-16 code points, including the required Unicode terminating null character.

Directory entries are grouped into blocks to form directory sectors. There are four directory
entries in a 512-byte directory sector (version 3 compound file), and there are 32 directory entries
in a 4,096- byte directory sector (version 4 compound file). The number of directory entries can
exceed the number of storage objects and stream objects due to unallocated directory entries.

[Child ID]: Entry::child_id
[Right Sibling ID]: Entry::right_sibling_id
[Left Sibling ID]: Entry::left_sibling_id
 */
#[derive(Debug)]
pub enum Entry<'a> {
    RootStorage(RootStorage<'a>),
    Storage(Storage<'a>),
    Stream(Stream<'a>),
    Unknown,
}

impl<'a> Entry<'a> {
    pub(crate) const LENGTH: u32 = 128;
    /// This field MUST be 0x00, 0x01, 0x02, or 0x05, depending on the actual type of object. All
    /// other values are not valid.
    pub(crate) fn object_type(offset: u64, file: &'a File) -> Result<ObjectType, String> {
        let byte = file.read_sized(offset + Self::NAME + 2, u8::from_ne_bytes);
        byte.try_into()
    }
}

macro_rules! impl_for_prop {
    ($self:ident, $method_name:ident) => {
        match $self {
            Self::Stream(ref stream) => stream.$method_name(),
            Self::Storage(ref storage) => storage.$method_name(),
            Self::RootStorage(ref root_storage) => root_storage.$method_name(),
            _ => panic!("unexpected method call!")
        }
    };
}

impl<'a> CommonProps<'a> for Entry<'a> {
    fn new(offset: u64, file: &'a File) -> Result<Self, String> {
        let ret = match Self::object_type(offset, file)? {
            ObjectType::Stream => Self::Stream(Stream::new(offset, file)?),
            ObjectType::Storage => Self::Storage(Storage::new(offset, file)?),
            ObjectType::RootStorage => Self::RootStorage(RootStorage::new(offset, file)?),
            ObjectType::Unknown => Self::Unknown,
        };
        Ok(ret)
    }

    fn offset(&self) -> u64 {
        impl_for_prop!(self, offset)
    }

    fn file(&self) -> &File {
        impl_for_prop!(self, file)
    }

    fn name(&self) -> String {
        impl_for_prop!(self, name)
    }

    fn name_length(&self) -> u16 {
        impl_for_prop!(self, name_length)
    }

    fn color_flag(&self) -> ColorFlag {
        impl_for_prop!(self, color_flag)
    }

    fn left_sibling_id(&self) -> Option<StreamID> {
        impl_for_prop!(self, left_sibling_id)
    }

    fn right_sibling_id(&self) -> Option<StreamID> {
        impl_for_prop!(self, right_sibling_id)
    }

    fn child_id(&self) -> Option<StreamID> {
        impl_for_prop!(self, child_id)
    }
}

pub trait CommonProps<'a>: Sized {
    const NAME: u64 = 64;
    const CLSID: u64 = 16;
    const STATE_BITS: u64 = 4;
    const TIME: u64 = 8;

    /// Creates an entry from a base offset and the source file.
    fn new(offset: u64, file: &'a File) -> Result<Self, String>;

    fn offset(&self) -> u64;

    fn file(&self) -> &File;

    /**
    This field MUST contain a Unicode string for the storage or stream name encoded in UTF-16. The
    name MUST be terminated with a UTF-16 terminating null character. Thus, storage and stream names
    are limited to 32 UTF-16 code points, including the terminating null character. When locating an
    object in the compound file except for the root storage, the directory entry name is compared by
    using a special case-insensitive uppercase mapping, described in Red-Black Tree. The following
    characters are illegal and MUST NOT be part of the name: '/', '\', ':', '!'.
     */
    fn name(&self) -> String;

    /// This field MUST match the length of the [Directory Entry Name] Unicode string in bytes. The
    /// length MUST be a multiple of 2 and include the terminating null character in the count. This
    /// length MUST NOT exceed 64, the maximum size of the [Directory Entry Name] field.
    ///
    /// [Directory Entry Name]: Self::name
    fn name_length(&self) -> u16;

    /// This field MUST be 0x00 (red) or 0x01 (black). All other values are not valid.
    fn color_flag(&self) -> ColorFlag;

    /// This field contains the stream ID of the left sibling.
    ///
    /// If there is no left sibling, the field MUST be set to NOSTREAM (0xFFFFFFFF).
    fn left_sibling_id(&self) -> Option<StreamID>;

    /// This field contains the stream ID of the right sibling.
    ///
    /// If there is no right sibling, the field MUST be set to NOSTREAM (0xFFFFFFFF).
    fn right_sibling_id(&self) -> Option<StreamID>;

    /// This field contains the stream ID of a child object.
    ///
    /// If there is no right sibling, the field MUST be set to NOSTREAM (0xFFFFFFFF).
    fn child_id(&self) -> Option<StreamID>;
}
