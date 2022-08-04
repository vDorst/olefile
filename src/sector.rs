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

use super::error::Error;

impl<'ole> super::ole::Reader<'ole> {
    pub(crate) fn read_sector(&self, sector_index: u32) -> Result<&[u8], Error> {
        let sector_size = self.sec_size;
        let sector_index = usize::try_from(sector_index).expect("sector_index to big for usize");
        let offset = sector_size + sector_size * sector_index;
        let max_size = offset + sector_size;

        // println!("\t + read_sector: {sector_index} 0x{offset:x}");

        if self.body.len() >= max_size {
            let sector = &self.body[offset..max_size];
            Ok(sector)
        } else {
            println!(
                "body: {} , max_size {max_size} {max_size:8x}",
                self.body.len()
            );
            Err(Error::BadSizeValue("File is too short"))
        }
    }
}
