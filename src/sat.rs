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

use std::{self, vec::Vec};

use super::{ole::Reader, error::Error, constants, util::FromSlice};

impl<'ole> Reader<'ole> {
    pub(crate) fn build_sat(&mut self) -> Result<(), Error> {
        let sector_size = self.sec_size;
        let mut sec_ids = vec![constants::FREE_SECID_U32; sector_size / 4];
        let msat_entries = &self.main_sat;

        // println!("\t build_sat msat_en {}", msat_entries.len());

        if msat_entries.is_empty() {
            Err(Error::EmptyMasterSectorAllocationTable)
        } else {
            for &sector_index in msat_entries.iter() {
                //let sector_index = msat_entries[i];
                // println!("\t read idx {sector_index:8x} sec 0x{sector_index:8x}");
                if sector_index == constants::END_OF_CHAIN_SECID_U32 {
                    break;
                }
                if sector_index == constants::FREE_SECID_U32 {
                    break;
                }
                self.read_sat_sector(sector_index, &mut sec_ids)?;
                self.sat.extend_from_slice(&sec_ids);
            }
            self.build_ssat()?;
            self.build_dsat()?;
            Ok(())
        }
    }

    pub(crate) fn read_sat_sector(
        &self,
        sector_index: u32,
        sec_ids: &mut Vec<u32>,
    ) -> Result<(), Error> {
        let sector = self.read_sector(sector_index)?;

        for (i, data) in sector.chunks_exact(4).enumerate() {
            if i == sec_ids.capacity() {
                break;
            }
            sec_ids[i] = u32::from_slice(data);
        }

        Ok(())
    }

    pub(crate) fn build_chain_from_sat(&mut self, start: u32) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut sector_index = start;
        while sector_index != constants::END_OF_CHAIN_SECID_U32
            && sector_index != constants::FREE_SECID_U32
        {
            chain.push(sector_index);
            // println!("\t\tbuild_chain_from_sat 0x{sector_index:8x}");
            sector_index = self.sat[sector_index as usize];
        }

        chain
    }

    pub(crate) fn build_chain_from_ssat(&mut self, start: u32) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut sector_index = start;
        while sector_index != constants::END_OF_CHAIN_SECID_U32
            && sector_index != constants::FREE_SECID_U32
        {
            chain.push(sector_index);

            sector_index = self.ssat[sector_index as usize];
        }

        chain
    }

    pub(crate) fn build_ssat(&mut self) -> Result<(), Error> {
        let mut sec_ids = vec![constants::FREE_SECID_U32; self.sec_size / 4];

        let sector_index = self.ssat.remove(0);
        let chain = self.build_chain_from_sat(sector_index);

        for sector_index in chain {
            self.read_sat_sector(sector_index, &mut sec_ids)?;
            self.ssat.extend_from_slice(&sec_ids);
        }
        Ok(())
    }

    pub(crate) fn build_dsat(&mut self) -> Result<(), Error> {
        let sector_index = self.dir_sat.remove(0);
        let chain = self.build_chain_from_sat(sector_index);

        for sector_index in chain {
            self.dir_sat.push(sector_index);
        }

        Ok(())
    }
}
