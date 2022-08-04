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

pub(crate) const HEADER_SIZE: usize = 512;
pub(crate) const IDENTIFIER: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

pub(crate) const UID_SIZE: usize = 16;

pub(crate) const LITTLE_ENDIAN_IDENTIFIER: [u8; 2] = [0xFE, 0xFF];
pub(crate) const BIG_ENDIAN_IDENTIFIER: [u8; 2] = [0xFF, 0xFE];

pub(crate) const FREE_SECID: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
pub(crate) const FREE_SECID_U32: u32 = 0xFFFF_FFFF;
pub(crate) const END_OF_CHAIN_SECID_U32: u32 = 0xFFFF_FFFE;
pub(crate) const CONTAINS_FAT_SECTORS: u32 = 0xFFFF_FFFD;
pub(crate) const SECID_MAX: u32 = 0xFFFF_FFF9;

pub(crate) const U32_SIZE: usize = std::mem::size_of::<u32>();

pub(crate) const DIRECTORY_ENTRY_SIZE: usize = 128;
