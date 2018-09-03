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
    is_name_unknown: bool
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
        if !self.is_name_unknown && self.name_str_id != 0 {
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
            is_name_unknown: true
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
    pub fn get_str(&self, id: usize) -> Result<Option<&str>, Error> {
        match self {
            FtlStrings::Production(strings) => Self::get_str_prod(strings, id),
            FtlStrings::Test(strings) => Ok(strings.get(&id).map(|string| string.as_str()))
        }
    }

    /// This function is used for `FtlStrings::Production`. It checks to see
    /// if the string exists, and then creates a `CStr` from a pointer. It
    /// is assumed that the string has a null terminator. Then the `CStr` is
    /// converted into `&str`.
    fn get_str_prod(strings: &[libc::c_char], id: usize) -> Result<Option<&str>, Error> {
        if strings.get(id).is_some() {
            unsafe { CStr::from_ptr(&strings[id]) }
                .to_str()
                .map(Option::Some)
                .context(ErrorKind::SharedMemoryRead)
                .map_err(Error::from)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FtlStrings;
    use libc;
    use std::collections::HashMap;

    #[test]
    fn get_str_valid() {
        let mut data = HashMap::new();
        data.insert(0, "".to_owned());
        data.insert(1, "test".to_owned());
        let strings = FtlStrings::Test(&data);

        assert_eq!(strings.get_str(0).unwrap(), Some(""));
        assert_eq!(strings.get_str(1).unwrap(), Some("test"));
    }

    #[test]
    fn get_str_prod() {
        let strings: Vec<libc::c_char> = ['\0', 't', 'e', 's', 't', '\0']
            .iter()
            .map(|&c| c as libc::c_char)
            .collect();

        assert_eq!(FtlStrings::get_str_prod(&strings, 0).unwrap(), Some(""));
        assert_eq!(FtlStrings::get_str_prod(&strings, 1).unwrap(), Some("test"));
        assert_eq!(FtlStrings::get_str_prod(&strings, 6).unwrap(), None);
    }
}
