use std::fs::File;
use crate::cfb::directory::entry::{Entry, CommonProps};

pub(crate) mod entry;

/// The [directory entry] array is a structure that is used to contain information about the stream
/// and storage objects in a [compound file], and to maintain a tree-style containment structure.
///
/// The directory entry array is allocated as a standard chain of directory sectors within the
/// [FAT]. Each directory entry is identified by a nonnegative number that is called the stream ID.
/// The first sector of the directory sector chain MUST contain the root storage directory entry as
/// the first directory entry at stream ID 0.
///
/// [directory entry]: self::entry::Entry
/// [compound file]: crate::cfb::Cfb
/// [FAT]: crate::cfb::fat::Fat
#[derive(Debug)]
pub(crate) struct Directory<'a> {
    offset: u64,
    length: u32,
    file: &'a File,
}

impl<'a> Directory<'a> {
    pub(crate) fn new(offset: u64, byte_count: u32, file: &'a File) -> Self {
        Self {
            offset,
            length: byte_count / Entry::LENGTH,
            file,
        }
    }

    pub(crate) fn entry(&self, index: u32) -> <Iter<'a> as Iterator>::Item {
        Entry::new(self.offset + (index * Entry::LENGTH) as u64, self.file)
    }

    pub(crate) fn len(&self) -> u32 {
        self.length
    }
}

impl<'a> IntoIterator for Directory<'a> {
    type Item = <Iter<'a> as Iterator>::Item;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

pub(crate) struct Iter<'a> {
    cursor: u32,
    directory: Directory<'a>,
}

impl<'a> Iter<'a> {
    pub(crate) fn new(directory: Directory<'a>) -> Self {
        Self {
            cursor: 0,
            directory,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Result<Entry<'a>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.directory.len() {
            return None;
        }

        let item = self.directory.entry(self.cursor);
        self.cursor += 1;
        Some(item)
    }
}

/// Each [directory entry] is identified by a nonnegative number that is called the stream ID.
///
/// [directory entry]: self::entry::Entry
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StreamID(u32);