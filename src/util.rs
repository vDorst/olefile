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

pub(crate) trait FromSlice<T> {
    fn from_slice(buf: &[T]) -> Self;
}

impl FromSlice<u8> for usize {
    fn from_slice(buf: &[u8]) -> Self {
        usize::try_from(u32::from_slice(buf)).expect("Your platform usize is smaller than u32!")
    }
}

impl FromSlice<u8> for u32 {
    fn from_slice(buf: &[u8]) -> Self {
        u32::from_le_bytes(buf.try_into().expect("Incorrect length"))
    }
}

impl FromSlice<u8> for u16 {
    fn from_slice(buf: &[u8]) -> Self {
        u16::from_le_bytes(buf.try_into().expect("Incorrect length"))
    }
}

impl FromSlice<u8> for i32 {
    fn from_slice(buf: &[u8]) -> Self {
        i32::from_le_bytes(buf.try_into().expect("Incorrect length"))
    }
}

impl FromSlice<u8> for u64 {
    fn from_slice(buf: &[u8]) -> Self {
        u64::from_le_bytes(buf.try_into().expect("Incorrect length"))
    }
}
