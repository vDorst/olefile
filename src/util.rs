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
        let mut result = 0usize;
        for (p, byte) in buf.iter().enumerate() {
            result += (*byte as usize) * 256usize.pow(p as u32);
        }
        result
    }
}

impl FromSlice<u8> for u32 {
    fn from_slice(buf: &[u8]) -> Self {
        let mut result = 0u32;
        for (p, byte) in buf.iter().enumerate() {
            result += (*byte as u32) * 256u32.pow(p as u32);
        }
        result
    }
}

impl FromSlice<u8> for i32 {
    fn from_slice(buf: &[u8]) -> Self {
        let mut result = 0i32;
        for (p, byte) in buf.iter().enumerate() {
            result += (*byte as i32) * 256i32.pow(p as u32);
        }
        result
    }
}

impl FromSlice<u8> for u64 {
    fn from_slice(buf: &[u8]) -> Self {
        let mut result = 0u64;
        for (p, byte) in buf.iter().enumerate() {
            result += (*byte as u64) * 256u64.pow(p as u32);
        }
        result
    }
}
