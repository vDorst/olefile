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

use super::{entry::Entry, ole::Reader};

/// Iterator for entries inside an OLE file.
pub struct OLEIterator<'a> {
    ole: &'a Reader<'a>,
    curr: usize,
}

impl<'a> OLEIterator<'a> {
    pub(crate) fn new(ole: &'a Reader) -> OLEIterator<'a> {
        OLEIterator { ole, curr: 0 }
    }
}

impl<'a> Iterator for OLEIterator<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<&'a Entry> {
        let entries = self.ole.entries.as_ref().unwrap();
        if self.curr < entries.len() {
            let entry = &entries[self.curr];
            self.curr += 1;
            Some(entry)
        } else {
            None
        }
    }
}
