// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Structs
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use failure::ResultExt;
use libc;
use shmem::Array;
use std::collections::HashMap;
use std::ffi::CStr;
use util::{Error, ErrorKind};

/// Used by FTL to check memory integrity in various structs
const MAGIC_BYTE: libc::c_uchar = 0x57;

/// The client struct stored in shared memory.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FtlClient {
    magic: libc::c_uchar,
    query_count: libc::c_int,
    blocked_count: libc::c_int,
    ip_str_id: libc::c_ulonglong,
    name_str_id: libc::c_ulonglong,
    is_name_resolved: bool
}

impl FtlClient {
    pub fn query_count(&self) -> usize {
        self.query_count as usize
    }

    pub fn blocked_count(&self) -> usize {
        self.blocked_count as usize
    }

    pub fn ip_str_id(&self) -> usize {
        self.ip_str_id as usize
    }

    pub fn name_str_id(&self) -> Option<usize> {
        // Only share the name if it exists and is not an empty string
        if self.is_name_resolved && self.name_str_id != 0 {
            Some(self.name_str_id as usize)
        } else {
            None
        }
    }
}

impl Default for FtlClient {
    fn default() -> Self {
        FtlClient {
            magic: MAGIC_BYTE,
            query_count: 0,
            blocked_count: 0,
            ip_str_id: 0,
            name_str_id: 0,
            is_name_resolved: false
        }
    }
}

/// A safe wrapper around FTL's strings. It is used to access the strings
/// referenced by other shared memory structs.
pub enum FtlStrings<'test> {
    Production(Array<libc::c_char>),
    Test(&'test HashMap<usize, String>)
}

impl<'test> FtlStrings<'test> {
    /// Read a string from FTL's string memory. If the string does not exist,
    /// `None` is returned. The `id` is the position of the string in
    /// shared memory, which can be obtained from the other shared memory
    /// structs.
    pub fn get_str(&self, id: usize) -> Option<Result<&str, Error>> {
        match self {
            FtlStrings::Production(strings) => {
                // This checks to see if the string exists, and then creates a
                // `CStr` from a pointer. It is assumed that the string has a
                // null terminator. Then the `CStr` is converted into `&str`.
                strings
                    .get(id)
                    .map(|_| unsafe { CStr::from_ptr(&strings[id]) })
                    .map(|msg| {
                        msg.to_str()
                            .context(ErrorKind::SharedMemoryRead)
                            .map_err(Error::from)
                    })
            }
            FtlStrings::Test(strings) => strings.get(&id).map(|string| Ok(string.as_str()))
        }
    }
}
