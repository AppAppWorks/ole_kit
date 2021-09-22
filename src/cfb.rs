pub mod header;
mod fat;
mod directory;

pub use fat::sector_number::SectorNumber as SectorNumber;

use std::fs::File;
use crate::cfb::header::{Header, SectorCount};
use core::fmt;
use std::fmt::Formatter;
use crate::cfb::fat::Fat;
use std::os::unix::fs::FileExt;
use crate::cfb::directory::Directory;
use crate::cfb::directory::entry::Entry;
use crate::cfb::directory::entry::CommonProps;

/**
A compound file is a structure that is used to store a hierarchy of storage objects and stream
objects into a single file or memory buffer.

A storage object is analogous to a file system directory. Just as a directory can contain other
directories and files, a storage object can contain other storage objects and stream objects. Also
like a directory, a storage object tracks the locations and sizes of the child storage object and
stream objects that are nested beneath it.

A stream object is analogous to the traditional notion of a file. Like a file, a stream contains
user-defined data that is stored as a consecutive sequence of bytes.

The hierarchy is defined by a parent object/child object relationship. Stream objects cannot contain
child objects. Storage objects can contain stream objects and/or other storage objects, each of
which has a name that uniquely identifies it among the child objects of its parent storage object.

The root storage object has no parent object. The root storage object also has no name. Because
names are used to identify child objects, a name for the root storage object is unnecessary and the
file format does not provide a representation for it.
 */
pub struct Cfb {
    file: File,
    sector_size: u32,
}

impl Cfb {
    /// Creates a compound file by reading the file at the path
    pub fn from_path(path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let header = Header::new(&file);
        let sector_size = 1 << header.sector_shift().0;

        Ok(Self { file, sector_size })
    }

    /// Returns the sector size in bytes of the compound file
    pub fn sector_size(&self) -> u32 {
        self.sector_size
    }

    /// Returns the header of the compound file
    pub fn header(&self) -> Header {
        Header::new(&self.file)
    }
}

impl Cfb {
    /// Returns a FAT structure by its sector number
    pub(crate) fn fat(&self, sector_no: SectorNumber) -> Fat {
        Fat::new(self.sector_bytes(sector_no))
    }

    /// Returns a FAT structure by the sector number of a stream object
    pub(crate) fn fat_by_stream_sector_no(&self, stream_sector_no: SectorNumber) -> Fat {
        let no_of_sectors_per_fat = SectorCount(self.sector_size >> 2);
        let difat_idx = stream_sector_no / no_of_sectors_per_fat;
        self.fat(self.header().sector_no_of_fat(difat_idx))
    }

    /// Returns a mini-FAT structure by its sector number
    pub(crate) fn mini_fat(&self, sector_no: SectorNumber) -> Fat {
        let sector_no_count = SectorCount(self.sector_size >> std::mem::size_of::<SectorNumber>());

        let min_fat_idx = sector_no / sector_no_count;

        let mut mini_fat_sector_number = self.header().first_mini_fat_sector_location();

        for _ in 0..min_fat_idx.0 {
            let fat = self.fat_by_stream_sector_no(mini_fat_sector_number);
            mini_fat_sector_number = fat
                .sector_number((mini_fat_sector_number % sector_no_count).0);
        }

        Fat::new(self.sector_bytes(mini_fat_sector_number))
    }

    /// Returns a mini-FAT structure by the sector number of a stream object
    pub(crate) fn mini_fat_by_stream_sector_no(&self, stream_sector_no: SectorNumber) -> Fat {
        let no_of_sectors_per_fat = SectorCount(self.sector_size >> 2);
        let mini_fat_idx = stream_sector_no / no_of_sectors_per_fat;
        self.mini_fat(mini_fat_idx)
    }

    /// Gets the bytes of a sector by its sector number
    #[inline]
    pub(crate) fn sector_bytes(&self, sector_no: SectorNumber) -> Vec<u8> {
        let mut bytes = vec![0u8; self.sector_size as usize];
        self.file.read_at(&mut bytes,
                          (sector_no + 1).byte_offset(self.sector_size));
        bytes
    }

    /// Returns a directory structure by its index
    pub(crate) fn directory(&self, index: u32) -> Directory {
        Directory::new((self.header().first_directory_sector_location() + index + 1).byte_offset(self.sector_size),
                       self.sector_size,
                       &self.file)
    }

    /// Returns an iterator over all directories of the compound file
    pub(crate) fn directories(&self) -> Iter {
        let first_directory_sector_location = self.header().first_directory_sector_location();
        Iter::new(first_directory_sector_location, self)
    }

    /// Gets a directory entry by its name, returns None if not found
    pub fn directory_entry(&self, name: &str) -> Option<Entry> {
        self.directories()
            .filter_map(|dir| {
                dir
                    .into_iter()
                    .filter_map(Result::ok)
                    .find(|entry| entry.name() == name)
            })
            .next()
    }

    /// Read the bytes of a stream object by its name, returns None if not found
    pub fn stream_bytes(&self, name: &str) -> Option<Vec<u8>> {
        self.directory_entry(name).and_then(|entry|
            match entry {
                Entry::Stream(stream) =>
                    Some(stream.stream_bytes(self, None)),
                Entry::RootStorage(root_storage) =>
                    Some(root_storage.mini_stream_bytes(self)),
                _ => None,
            }
        )
    }

    /// Read the bytes of the mini stream
    pub fn mini_stream_bytes(&self) -> Vec<u8> {
        self.directories()
            .next()
            .map(|dir| dir.entry(0))
            .map(|entry|
                match entry {
                    Ok(Entry::RootStorage(root)) =>
                        root.mini_stream_bytes(&self),
                    _ => panic!("impossible"),
                }
            ).unwrap()
    }
}

impl fmt::Debug for Cfb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_map();
        crate::debug_map_method_reflection!(fmt, self, sector_size, header);
        fmt.finish()
    }
}

pub(crate) struct Iter<'a> {
    next_sector: SectorNumber,
    cfb: &'a Cfb,
}

impl<'a> Iter<'a> {
    pub(crate) fn new(next_sector: SectorNumber, cfb: &'a Cfb) -> Self {
        Self {
            next_sector,
            cfb,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Directory<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.next_sector.is_other() {
            return None
        }

        let sector = self.next_sector;

        let fat = self.cfb.fat_by_stream_sector_no(sector);
        self.next_sector = fat.sector_number(sector.0 % (self.cfb.sector_size >> std::mem::size_of::<SectorNumber>()));

        let dir = Some(Directory::new((sector + 1).byte_offset(self.cfb.sector_size),
                            self.cfb.sector_size,
                            &self.cfb.file));
        dir
    }
}
