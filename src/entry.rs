//             DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyright (C) 2018 Thomas Bailleux <thomas@bailleux.me>
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.
//
// Author: zadig <thomas chr(0x40) bailleux.me>

use super::{constants, error::Error, ole::Reader};
use std::{fmt, string::String, vec::Vec};

#[derive(Debug)]
pub(crate) enum NodeColour {
    Red,
    Black,
}

impl NodeColour {
    fn from(t: u8) -> Result<NodeColour, Error> {
        match t {
            0 => Ok(NodeColour::Red),
            1 => Ok(NodeColour::Black),
            _ => Err(Error::NodeTypeUnknown),
        }
    }
}

impl fmt::Display for NodeColour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NodeColour::Red => write!(f, "RED"),
            NodeColour::Black => write!(f, "BLACK"),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum EntryType {
    /// Empty entry.
    Empty,

    /// Storage, i.e. a directory.
    UserStorage,

    /// Stream, i.e. a file.
    UserStream,

    /// LockBytes (unknown usage).
    LockBytes,

    /// Property (unknown usage).
    Property,

    /// Root storage.
    RootStorage,
}

impl EntryType {
    fn from(t: u8) -> Result<EntryType, Error> {
        match t {
            0 => Ok(EntryType::Empty),
            1 => Ok(EntryType::UserStorage),
            2 => Ok(EntryType::UserStream),
            3 => Ok(EntryType::LockBytes),
            4 => Ok(EntryType::Property),
            5 => Ok(EntryType::RootStorage),
            _ => Err(Error::NodeTypeUnknown),
        }
    }
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EntryType::Empty => write!(f, "Empty"),
            EntryType::UserStorage => write!(f, "User storage"),
            EntryType::UserStream => write!(f, "User stream"),
            EntryType::LockBytes => write!(f, "?? Lock bytes ??"),
            EntryType::Property => write!(f, "?? Property ??"),
            EntryType::RootStorage => write!(f, "Root storage"),
        }
    }
}

/// An entry in an OLE file.
///
/// An entry means a stream or a storage.
/// A stream is a file, and a storage is a folder.
///
/// # Basic Example
///
/// ```
/// use ole::Reader;
///
/// let mut parser =
///       Reader::from_path("assets/Thumbs.db").unwrap();
///
/// let entry = parser.iterate().next().unwrap();
/// println!("Name of the entry: {}", entry.name());
/// println!("Type of the entry: {}", entry._type());
/// println!("Size of the entry: {}", entry.len());
/// ```

pub struct Entry {
    /// ID of the entry.
    id: u32,

    /// Name of the stream or the storage.
    name: String,

    /// Type of the entry.
    entry_type: EntryType,

    /// Color of the entry (see <https://en.wikipedia.org/wiki/Red%E2%80%93black_tree>)
    color: NodeColour,

    /// ID of the left child entry.
    left_child_node: u32,

    /// ID of the right child entry.
    right_child_node: u32,

    /// ID of the root node
    root_node: u32,

    /// UID of the entry.
    identifier: [u8; 16], // 16 bytes

    /// Flags of the entry.
    flags: u32, // 4 bytes

    /// Creation time.
    creation_time: u64,

    /// Last modification time.
    last_modification_time: u64,

    /// Chain of secID which hold the stream or the storage
    sec_id_chain: Vec<u32>,

    /// Size of the entry.
    size: usize,

    /// Array of the children's DirIDs
    children_nodes: Vec<u32>,

    /// DirID of the parent
    parent_node: Option<u32>,
}

impl Entry {
    fn from_slice(sector: &[u8], dir_id: u32) -> Result<Entry, Error> {
        use crate::util::FromSlice;
        let entry = Entry {
            id: dir_id,
            name: Entry::build_name(&sector[0..64]),
            entry_type: EntryType::from(sector[66])?,
            color: NodeColour::from(sector[67])?,
            left_child_node: u32::from_slice(&sector[68..72]),
            right_child_node: u32::from_slice(&sector[72..76]),
            root_node: u32::from_slice(&sector[76..80]),
            identifier: {
                let mut identifier = [0u8; 16];
                identifier.clone_from_slice(&sector[80..96]);
                identifier
            },
            flags: u32::from_slice(&sector[96..100]),
            creation_time: u64::from_slice(&sector[100..108]),
            last_modification_time: u64::from_slice(&sector[108..116]),
            sec_id_chain: vec![u32::from_slice(&sector[116..120])],
            size: usize::from_slice(&sector[120..124]),
            children_nodes: Vec::new(),
            parent_node: None,
        };

        Ok(entry)
    }

    fn build_name(array: &[u8]) -> String {
        let mut name = String::new();

        let mut i = 0usize;
        while i < 64 && array[i] != 0 {
            name.push(array[i] as char);
            i += 2;
        }

        name
    }

    /// Returns the ID of the entry.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the creation time of the entry (could be 0)
    pub fn creation_time(&self) -> u64 {
        self.creation_time
    }

    /// Returns the last modification time of the entry (could be 0)
    pub fn last_modification_time(&self) -> u64 {
        self.last_modification_time
    }

    /// Returns the name of the entry.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the type of the entry.
    pub fn _type(&self) -> EntryType {
        self.entry_type
    }

    /// Returns the size of the entry
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the `DirID` of the left child node
    pub fn left_child_node(&self) -> u32 {
        self.left_child_node
    }

    /// Returns the `DirID` of the right child node
    pub fn right_child_node(&self) -> u32 {
        self.right_child_node
    }

    /// Returns the `DirID` of the parent, if exists
    pub fn parent_node(&self) -> Option<u32> {
        self.parent_node
    }

    /// Returns the `DirID`s of the children, if exists
    pub fn children_nodes(&self) -> &Vec<u32> {
        &self.children_nodes
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Entry #{}. Type: {}, Color: {}, Name: {},
      Size: {}. SecID chain: {:?}",
            self.id, self.entry_type, self.color, &self.name, self.size, self.sec_id_chain
        )
    }
}

/// Slice of the content of the entry.
///
/// This is not an ordinary slice, because OLE files are like FAT system:
/// they are based on sector and SAT. Therefore, a stream can be fragmented
/// through the file.
///
/// # Basic example
///
/// ```
/// use ole::Reader;
/// use std::io::{Read, Seek};
/// let mut parser =
///       Reader::from_path("assets/Thumbs.db").unwrap();
///
/// let entry = parser.iterate().next().unwrap();
/// let mut slice = parser.get_entry_slice(entry).unwrap();
/// // Read the first 42 bytes of the entry;
/// let mut buf = [0u8; 42];
/// let nread = slice.read(&mut buf).unwrap();
///
/// ```
pub struct EntrySlice<'s> {
    /// Chunk size, i.e. size of the sector.
    max_chunk_size: usize,

    /// List of slices.
    chunks: Vec<&'s [u8]>,

    /// How many bytes which have been already read.
    read: usize,

    /// Total size of slice.
    total_size: usize,

    /// Real size of all chunks
    real_size: usize,
}

impl<'s> EntrySlice<'s> {
    fn new(max_chunk_size: usize, size: usize) -> EntrySlice<'s> {
        EntrySlice {
            max_chunk_size,
            chunks: Vec::new(),
            read: 0usize,
            total_size: size,
            real_size: 0,
        }
    }

    fn add_chunk(&mut self, chunk: &'s [u8]) {
        self.real_size += chunk.len();
        self.chunks.push(chunk);
    }

    /// Returns the length of the slice, therefore the length of the entry.
    pub fn len(&self) -> usize {
        self.total_size
    }

    pub fn is_empty(&self) -> bool {
        self.total_size == 0
    }

    /// Returns the real length of all chunks
    pub fn real_len(&self) -> usize {
        self.real_size
    }
}

impl<'s> std::io::Read for EntrySlice<'s> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let to_read = std::cmp::min(buf.len(), self.total_size - self.read);
        let result: Result<usize, std::io::Error>;
        if to_read == 0 {
            result = Ok(0usize);
        } else {
            let mut offset = self.read;
            let mut read = 0;
            while read != to_read {
                let chunk_index = offset / self.max_chunk_size;
                if chunk_index >= self.chunks.len() {
                    break;
                }
                let chunk = &self.chunks[chunk_index];
                let local_offset = offset % self.max_chunk_size;
                let end = std::cmp::min(local_offset + to_read - read, self.max_chunk_size);
                let slice = &chunk[local_offset..end];
                for u in slice {
                    buf[read] = *u;
                    read += 1;
                    self.read += 1;
                }
                offset = self.read;
            }
            result = Ok(read);
        }

        result
    }
}

impl<'ole> Reader<'ole> {
    /// Returns the slice for the entry.
    pub fn get_entry_slice(&self, entry: &Entry) -> Result<EntrySlice, Error> {
        let entry_slice: EntrySlice;
        let size = entry.size;
        if size == 0 {
            Err(Error::EmptyEntry)
        } else {
            if size < self.minimum_standard_stream_size {
                entry_slice = self.get_short_stream_slices(&entry.sec_id_chain, size)?;
            } else {
                entry_slice = self.get_stream_slices(&entry.sec_id_chain, size)?;
            }
            Ok(entry_slice)
        }
    }

    pub(crate) fn build_directory_entries(&mut self) -> Result<(), Error> {
        let n_entry_by_sector = self.sec_size / constants::DIRECTORY_ENTRY_SIZE;
        let mut entries = Vec::<Entry>::with_capacity(self.dir_sat.len() * n_entry_by_sector);

        let mut k = 0u32;
        for i in 0..self.dir_sat.len() {
            let sector_index = self.dir_sat[i];
            let sector = self.read_sector(sector_index)?;

            // self.dump_difat_sector(&sector);

            let loc = i * constants::U32_SIZE;
            if sector[loc..loc + 4] == constants::FREE_SECID {
                break;
            }

            for l in 0..n_entry_by_sector {
                let entry = Entry::from_slice(
                    &sector[l * constants::DIRECTORY_ENTRY_SIZE
                        ..(l + 1) * constants::DIRECTORY_ENTRY_SIZE],
                    k,
                )?;
                entries.push(entry);
                k += 1;
            }
        }
        let stream_size = self.minimum_standard_stream_size;

        for (i, entry) in (0_u32..).zip(&mut entries.iter_mut()) {
            match entry.entry_type {
                EntryType::UserStream => {
                    let start_index = entry.sec_id_chain.pop().expect("No sectors in cahin!");
                    entry.sec_id_chain = if entry.size < stream_size {
                        self.build_chain_from_ssat(start_index)
                    } else {
                        self.build_chain_from_sat(start_index)
                    };
                }
                EntryType::RootStorage => {
                    self.root_entry = Some(i);
                    let start_index = entry.sec_id_chain.pop().expect("No sectors in cahin!");
                    entry.sec_id_chain = self.build_chain_from_sat(start_index);
                }
                _ => {}
            }
        }
        self.entries = Some(entries);
        self.build_entry_tree(0, None);
        Ok(())
    }

    fn get_short_stream_slices(&self, chain: &Vec<u32>, size: usize) -> Result<EntrySlice, Error> {
        let short_sector_size = self.short_sec_size;
        let mut entry_slice = EntrySlice::new(short_sector_size, size);
        let short_stream_chain = &self.entries.as_ref().unwrap()[0].sec_id_chain.clone();
        let n_per_sector = self.sec_size / short_sector_size;
        let mut total_read = 0;
        for short_sec_id in chain {
            let sector_index = short_stream_chain[*short_sec_id as usize / n_per_sector];
            let sector = self.read_sector(sector_index)?;
            let short_sector_index = *short_sec_id as usize % n_per_sector;
            let start = short_sector_index as usize * short_sector_size;
            let end = start + std::cmp::min(short_sector_size, size - total_read);
            entry_slice.add_chunk(&sector[start..end]);
            total_read += end - start;
        }
        Ok(entry_slice)
    }

    fn get_stream_slices(&self, chain: &Vec<u32>, size: usize) -> Result<EntrySlice, Error> {
        let sector_size = self.sec_size;
        let mut entry_slice = EntrySlice::new(sector_size, size);
        let mut total_read = 0;
        for sector_id in chain {
            let sector = self.read_sector(*sector_id)?;
            let start = 0usize;
            let end = std::cmp::min(sector_size, size - total_read);
            entry_slice.add_chunk(&sector[start..end]);
            total_read += end - start;
        }
        Ok(entry_slice)
    }

    fn build_entry_tree(&mut self, id: u32, parent_id: Option<u32>) {
        // println!("build_entry_tree 0x{id:8x}");
        if id != constants::FREE_SECID_U32 {
            let (child, node_type, n) = {
                let entries = self.entries.as_mut().expect("Valid entry");

                let n = u32::try_from(entries.len()).expect("length to big to fit in u32");

                // if id >= n {
                //     panic!("read out of bouned.");
                // }

                // Register as child
                if let Some(parent_id) = parent_id {
                    entries[parent_id as usize].children_nodes.push(id);
                }

                let entry = &mut entries[id as usize];

                // Register the parent id for the current node
                entry.parent_node = parent_id;

                (entry.root_node, entry._type(), n)
            };

            if node_type == EntryType::RootStorage || node_type == EntryType::UserStorage {
                self.build_entry_tree(child, Some(id));
            }

            let entry = &mut self.entries.as_mut().unwrap()[id as usize];

            let left_child = entry.left_child_node();
            let right_child = entry.right_child_node();

            if left_child < n {
                self.build_entry_tree(left_child, parent_id);
            }
            if right_child < n {
                self.build_entry_tree(right_child, parent_id);
            }
        }
    }
}
