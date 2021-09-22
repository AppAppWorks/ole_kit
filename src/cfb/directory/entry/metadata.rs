use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ObjectType {
    Unknown = 0x00,
    /// An object in a compound file that is analogous to a file system directory. The parent object
    /// of a storage object must be another storage object or the root storage object.
    Storage = 0x01,
    /// An object in a compound file that is analogous to a file system file. The parent object of a
    /// stream object must be a storage object or the root storage object.
    Stream = 0x02,
    /// A storage object in a compound file that must be accessed before any other storage objects
    /// and stream objects are referenced. It is the uppermost parent object in the storage object
    /// and stream object hierarchy.
    RootStorage = 0x05,
}

impl TryFrom<u8> for ObjectType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Unknown as u8 => Ok(Self::Unknown),
            x if x == Self::Storage as u8 => Ok(Self::Storage),
            x if x == Self::Stream as u8 => Ok(Self::Stream),
            x if x == Self::RootStorage as u8 => Ok(Self::RootStorage),
            _ => Err(format!("invalid value({}) for ObjectType!", value)),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ColorFlag {
    Red,
    Black,
}

impl TryFrom<u8> for ColorFlag {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Red as u8 => Ok(Self::Red),
            x if x == Self::Black as u8 => Ok(Self::Black),
            _ => Err("invalid value for entry::metadata::ColorFlag!"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StreamSize(pub u64);