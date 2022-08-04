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

use std::vec::Vec;

use super::{constants, error::Error, ole::Reader, util::FromSlice};

impl<'ole> Reader<'ole> {
    pub(crate) fn parse_header(&mut self) -> Result<(), Error> {
        // read the header
        //let mut header_sector_data: std::vec::Vec<u8> = vec![0u8; super::constants::HEADER_SIZE];
        // let s = self.read(&mut header_sector_data)?;

        let header_sector_data = self.body[0..constants::HEADER_SIZE].to_vec();

        // // Check file header size
        // if s != super::constants::HEADER_SIZE {
        //     return Err(super::error::Error::BadFileSize);
        // }

        // Check file identifier
        if constants::IDENTIFIER != header_sector_data[0..8] {
            return Err(Error::InvalidOLEFile);
        }

        //self.dump_header(&header_sector_data);

        // UID
        self.uid.copy_from_slice(&header_sector_data[8..24]);

        // Revision number & version number
        let rv_number = u16::from_slice(&header_sector_data[24..26]);
        self.revision_number = rv_number;

        let rv_number = u16::from_slice(&header_sector_data[26..28]);
        self.version_number = rv_number;

        if !(3..4).contains(&rv_number) {
            return Err(Error::InvalidOLEVersion(self.version_number));
        }

        if self.revision_number != 0x003e {
            return Err(Error::InvalidOLEVersion(self.revision_number));
        }

        // println!("OLE Version {} {:x}", self.version_number, self.revision_number);

        // Check little-endianness; big endian not yet supported
        if header_sector_data[28..30] == constants::BIG_ENDIAN_IDENTIFIER {
            return Err(Error::NotImplementedYet);
        }

        if header_sector_data[28..30] != constants::LITTLE_ENDIAN_IDENTIFIER {
            return Err(Error::InvalidOLEFile);
        }

        // println!("HIER2");

        // Sector size or Sector Shift
        let mut k = u32::from(u16::from_slice(&header_sector_data[30..32]));

        // if k >= 16, it means that the sector size equals 2 ^ k, which
        // is impossible.
        if k >= 16 {
            return Err(Error::BadSizeValue("Overflow on sector size"));
        }
        if self.revision_number == 0x3 && k != 0x9 {
            return Err(Error::BadSizeValue("Wrong Sector size!"));
        }
        if self.revision_number == 0x4 && k != 0xC {
            return Err(Error::BadSizeValue("Wrong Sector size!"));
        }

        self.sec_size = 2usize.pow(k);

        // Short sector size
        k = u32::from(u16::from_slice(&header_sector_data[32..34]));

        // same for sector size
        if k >= 16 {
            return Err(Error::BadSizeValue("Overflow on short sector size"));
        }
        if self.revision_number == 0x4 && k != 0x6 {
            return Err(Error::BadSizeValue("Wrong Mini Sector size!"));
        }

        self.short_sec_size = 2usize.pow(k);

        let alloc_size = (self.short_sec_size / constants::U32_SIZE)
            * usize::from_slice(&header_sector_data[44..48]);

        // Total number of sectors used for the sector allocation table
        let total_sector_alloc_table = Vec::<u32>::with_capacity(alloc_size);

        // SecID of the first sector of directory stream and Read DIFAT Start Sector Location
        let difat_sector_alloc_table: Vec<u32> = vec![u32::from_slice(&header_sector_data[48..52])];

        // Minimum size of a standard stream (bytes)
        self.minimum_standard_stream_size = usize::from_slice(&header_sector_data[56..60]);

        // standard says that this value has to be greater
        // or equals to 4096
        if self.minimum_standard_stream_size < 4096usize {
            return Err(Error::InvalidOLEFile);
        }

        // println!("HIER4");

        // secID of the first sector of the SSAT & Total number
        // of sectors used for the short-sector allocation table
        let mut ssat = Vec::with_capacity(
            usize::from_slice(&header_sector_data[64..68]) * (self.sec_size / 4),
        );
        ssat.push(u32::from_slice(&header_sector_data[60..64]));

        // secID of first sector of the master sector allocation table
        // & Total number of sectors used for
        // the master sector allocation table

        let difat_sectors = usize::from_slice(&header_sector_data[72..76]);

        let msat_size = 109
            + if difat_sectors == constants::END_OF_CHAIN_SECID_U32 as usize {
                0
            } else {
                difat_sectors
            };

        self.main_sat = Vec::with_capacity(msat_size);

        self.sat = total_sector_alloc_table;
        self.dir_sat = difat_sector_alloc_table;
        self.ssat = ssat;

        // println!("HIER5");

        // now we build the MSAT
        self.build_master_sector_allocation_table(&header_sector_data)?;
        Ok(())
    }

    /// Dump Header
    pub fn dump_header(&self, header: &[u8]) {
        for (e, data) in header.chunks(4).enumerate() {
            let d = u32::from_slice(data);

            let byte = e * 4;
            match byte {
                0 => println!("SIGNATURE"),
                16 => println!("CLSID"),
                48 => println!("Dir Start Sector"),
                68 => {
                    println!("Fist DIFAT ector Location");
                    let loc = d.wrapping_mul(512);
                    print!("\t{e:3} - {byte:3}: 0x{d:8x} [{d}] LOC: 0x{loc:8x} [{loc}] -- ");
                }
                72 => {
                    println!("Number of DIFAT");
                }
                76 => println!("DIFAT"),
                _ => (),
            }

            println!("\t{e:3} - {byte:3}: 0x{d:8x} [{d}]");
        }
    }

    /// Dump sector
    pub fn dump_difat_sector(&self, header: &[u8]) {
        println!("\n\t\tDUMP DIFAT SECTOR");
        for (e, data) in header.chunks(4).enumerate() {
            let d: u32 = u32::from_slice(data);

            if (0..0xFFFF_FFF9).contains(&d) {
                println!("\t{e:3} - 0x{d:8x} [{d}]");
            } else if d == 0xFFFFFFFD {
                println!("\t{e:3} - FAT SECTOR 0x{d:8x}");
            } else if d == 0xFFFFFFFE {
                println!("\t{e:3} - ENDOFCHAIN 0x{d:8x}");
            } else if d == 0xFFFFFFFF {
                println!("\t{e:3} - FREE 0x{d:8x}");
            } else {
                println!("\t{e:3} - UNKNOWN 0x{d:8x}");
            }
        }
    }

    /// Build the Master Sector Allocation Table (MSAT)
    fn build_master_sector_allocation_table(&mut self, header: &[u8]) -> Result<(), Error> {
        self.main_sat.clear();

        // First, we build the master sector allocation table from the header
        let mut total_sec_id_read = self.read_sec_ids(&header[76..]);

        // Check if additional sectors are used for building the msat
        if total_sec_id_read == 109 {
            // println!("total_sec_id_read {total_sec_id_read}");
            // return Err(super::error::Error::NotImplementedYet);
            let sec_size = self.sec_size;
            let mut sec_id = u32::from_slice(&header[68..72]);
            let number = usize::from_slice(&header[72..76]);

            for _ in 0..number {
                if sec_id == constants::END_OF_CHAIN_SECID_U32 {
                    break;
                }
                if sec_id == constants::FREE_SECID_U32 {
                    break;
                }

                // println!("DIFAT block {i}, sector 0x{sec_id:x} {sec_id}x");

                // // check if we need to read more data
                // if buffer.len() <= relative_offset + sec_size {
                //     let new_len = (sec_id + 1) * sec_size;
                //     println!("\tAlloc new buffer space {} -> {}", buffer.len(), new_len);
                //     buffer.resize(new_len, 0xEEu8);
                //     self.read(&mut buffer[relative_offset..relative_offset + sec_size])?;
                // }

                let buffer = self.read_sector(sec_id)?.to_vec();

                assert!(buffer.len() == sec_size);

                let b = &buffer;
                // self.dump_difat_sector(b);
                let found = self.read_sec_ids(&b[0..sec_size - 4]);

                total_sec_id_read += found;

                sec_id = u32::from_slice(&b[sec_size - 4..sec_size]);

                // println!("---- LAST SECID 0x{sec_id:8x}");

                if sec_id != constants::END_OF_CHAIN_SECID_U32
                    && sec_id != constants::FREE_SECID_U32
                {
                    panic!("Invalid DIFAT ending!");
                }
            }
        }

        // println!("Found total of {total_sec_id_read}");

        //self.main_sat.resize(total_sec_id_read, constants::FREE_SECID_U32);

        Ok(())
    }

    fn read_sec_ids(&mut self, buffer: &[u8]) -> usize {
        let mut i = 0usize;
        // let max_sec_ids = buffer.len() / 4;

        for data in buffer.chunks_exact(constants::U32_SIZE) {
            let secid = u32::from_slice(data);
            // if secid == constants::FREE_SECID_U32 {
            //     println!("EOE!");
            //     break;
            // }
            // if secid == constants::CONTAINS_FAT_SECTORS {
            //     println!("EOD!");
            //     break;
            // }
            // if secid > super::constants::SECID_MAX {
            //     println!("Unknown ID 0x{secid:8x}");
            //     break;
            // }

            // println!("\tsec_id found {i:3} idx {}, 0x{secid:8x} {secid} ", self.main_sat.len());

            if secid == constants::END_OF_CHAIN_SECID_U32 {
                break;
            }
            if secid == constants::FREE_SECID_U32 {
                break;
            }

            self.main_sat.push(secid);

            i += 1;
        }

        i
    }
}
