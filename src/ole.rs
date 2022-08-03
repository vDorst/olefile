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

use std::{
    self,
    io::{BufReader, Read},
    vec::Vec,
};

use super::{constants, entry::Entry, error::Error, iterator::OLEIterator};

/// An OLE file reader.
///
/// The parsing method follows the same method described here:
/// <http://www.openoffice.org/sc/compdocfileformat.pdf>
///
/// # Basic Example
///
/// ```
/// use ole::Reader;
///
/// let mut reader =
///       Reader::from_path("assets/Thumbs.db").unwrap();
///
/// println!("This OLE file stores the following entries: ");
/// for entry in reader.iterate() {
///   println!("{}", entry);
/// }
/// ```

pub struct Reader<'ole> {
    /// Buffer for reading from the source.
    pub(crate) buf_reader: BufReader<Box<dyn Read + 'ole>>,

    /// Unique identifier.
    pub(crate) uid: [u8; 16],

    /// Revision number.
    pub(crate) revision_number: u16,

    /// Version number.
    pub(crate) version_number: u16,

    /// Size of one sector.
    pub(crate) sec_size: usize,

    /// Size of one short sector.
    pub(crate) short_sec_size: usize,

    /// Sector Allocation Table.
    pub(crate) sat: Vec<u32>,

    /// Directory Sector Allocation Table.
    pub(crate) dir_sat: Vec<u32>,

    /// Minimum size of a standard stream size.
    pub(crate) minimum_standard_stream_size: usize,

    /// Short Sector Allocation Table.
    pub(crate) ssat: Vec<u32>,

    /// Master Sector Allocation Table.
    pub(crate) main_sat: Vec<u32>,

    /// Body of the file.
    pub(crate) body: Vec<u8>,

    /// Directory entries.
    pub(crate) entries: Option<Vec<Entry>>,

    /// DirID of the root entry.
    pub(crate) root_entry: Option<u32>,
}

impl<'ole> Reader<'ole> {
    /// Constructs a new `Reader`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ole;
    /// let mut my_resume = std::fs::File::open("assets/Thumbs.db").unwrap();
    /// let mut parser = ole::Reader::new(my_resume).unwrap();
    /// ```
    pub fn new<T: 'ole>(readable: T) -> std::result::Result<Reader<'ole>, Error>
    where
        T: Read,
    {
        let mut read: BufReader<Box<dyn Read>> = BufReader::new(Box::new(readable));

        let mut buf = Vec::<u8>::with_capacity(10_000_000);

        let body = read.read_to_end(&mut buf)?;

        println!("Readed file {body}");

        // panic!("bye");

        let mut t = Reader {
            buf_reader: read,
            uid: [0u8; constants::UID_SIZE],
            revision_number: 0,
            version_number: 0,
            sec_size: 0,
            short_sec_size: 0,
            sat: Vec::new(),
            dir_sat: Vec::new(),
            minimum_standard_stream_size: 0,
            ssat: Vec::new(),
            main_sat: vec![constants::FREE_SECID_U32; 109],
            body: buf,
            entries: None,
            root_entry: None,
        };
        println!("parse_header");
        t.parse_header()?;
        println!("build_sat");
        t.build_sat()?;
        println!("build_dir_entries");
        t.build_directory_entries()?;
        println!("Done");
        Ok(t)
    }

    /// Constructs a new `Reader` from a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use ole;
    /// let mut parser = ole::Reader::from_path("assets/Thumbs.db").unwrap();
    /// ```
    pub fn from_path(path: &str) -> Result<Reader, Error> {
        let f = std::fs::File::open(path).map_err(Error::IOError)?;
        Reader::new(f)
    }

    /// Returns an iterator for directory entries of the OLE file.
    ///
    /// # Examples
    ///
    /// ```
    /// use ole;
    /// let mut parser = ole::Reader::from_path("assets/Thumbs.db").unwrap();
    ///
    /// for entry in parser.iterate() {
    ///   println!("Entry {}", entry.name());
    /// }
    /// ```
    pub fn iterate(&self) -> OLEIterator {
        OLEIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{constants, Error, Reader};

    #[test]
    fn instance_nok() {
        let path = "Thumbs.db";
        let o: Result<Reader, Error> = Reader::from_path(path);
        assert_eq!(o.is_ok(), false);
        let e = o.err().unwrap();
        println!("NOK: {}", e);
    }

    #[test]
    fn instance_ok() {
        let path = "./assets/Thumbs.db";
        let o: Result<Reader, Error> = Reader::from_path(path);
        assert_eq!(o.is_ok(), true);
    }

    #[test]
    fn sector_sizes() {
        let ole: Reader = Reader::from_path("./assets/Thumbs.db").unwrap();
        assert_eq!(ole.sec_size, 512);
        assert_eq!(ole.short_sec_size, 64);
    }

    #[test]
    fn array_bad_identifier() {
        let mut vec = constants::IDENTIFIER.to_vec();
        vec[0] = 0xD1;
        fill(&mut vec);
        let ole = Reader::new(&vec[..]);
        assert_eq!(ole.is_ok(), false);
        println!("BAD IDENTIFIER: {}", ole.err().unwrap());
    }

    fn fill(buf: &mut std::vec::Vec<u8>) {
        let missing = vec![0u8; constants::HEADER_SIZE - buf.len()];
        buf.extend(missing);
    }

    #[test]
    fn array_bad_endianness_identifier() {
        let mut vec = constants::IDENTIFIER.to_vec();
        vec.extend(vec![0u8; 20]);
        vec.push(0xFE);
        vec.push(0xFE);
        fill(&mut vec);
        let ole = Reader::new(&vec[..]);
        match ole {
            Ok(_t) => assert!(false),
            Err(e) => println!("BAD ENDIANNESS: {}", e),
        }
    }

    #[test]
    fn uid() {
        let ole = Reader::from_path("./assets/Thumbs.db");
        assert_eq!(ole.is_ok(), true);
        let ole = ole.unwrap();
        assert_eq!(&[0x0u8; 16] == &ole.uid[..], true);
    }

    #[test]
    fn bad_sec_size() {
        let mut vec = constants::IDENTIFIER.to_vec();
        vec.extend(vec![0x42u8; 20]);
        vec.extend(&constants::LITTLE_ENDIAN_IDENTIFIER);
        vec.extend(vec![0xFF, 0xFF, 0xFF, 0xFF]);
        vec.extend(vec![0u8; 10]);
        vec.extend(vec![0xFF, 0xFF, 0xFF, 0xFF]);
        fill(&mut vec);
        let ole = Reader::new(&vec[..]);
        assert_eq!(ole.is_ok(), false);
    }

    #[test]
    fn several_values() {
        let ole = Reader::from_path("./assets/Thumbs.db").unwrap();
        assert_eq!(ole.sat.capacity(), 128usize);
        assert_eq!(ole.main_sat.len(), 1usize);
        assert_eq!(ole.ssat.capacity(), 512usize);
    }

    #[test]
    fn print_things() {
        let ole = Reader::from_path("./assets/sample.ppt").unwrap();
        println!("STREAM SIZE: {}", ole.minimum_standard_stream_size);
        println!("MSAT: {:?}", ole.main_sat);
        println!("SAT: {:?}", ole.sat);
        println!("SSAT: {:?}", ole.ssat);
        println!("DSAT: {:?}", ole.dir_sat);
        for entry in ole.iterate() {
            println!("{}", entry);
            // if let Ok(mut slice) = ole.get_entry_slice(entry) {
            //     let mut buf = vec![0u8; slice.len()];
            //     let read_size = slice.read(&mut buf).unwrap();
            //     let mut file =
            //         std::fs::File::create(format!("./assets/streams/{}.bin", entry.name()))
            //             .unwrap();
            //     println!("Real len: {}", slice.real_len());
            //     file.write_all(&buf).unwrap();
            //     assert_eq!(read_size, slice.real_len());
            //     assert_eq!(read_size, slice.len());
            // }
        }
    }
}
