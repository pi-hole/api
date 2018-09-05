// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Strings Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use libc;
use shmem::Array;
use std::collections::HashMap;
use std::ffi::CStr;

/// A safe wrapper around FTL's strings. It is used to access the strings
/// referenced by other shared memory structs.
///
/// Note: When testing, the 0 entry will be ignore in favor of returning the
/// empty string
pub enum FtlStrings<'test> {
    Production(Array<libc::c_char>),
    Test(&'test HashMap<usize, String>)
}

impl<'test> FtlStrings<'test> {
    /// Read a string from FTL's string memory. If the string does not exist,
    /// `None` is returned. The `id` is the position of the string in
    /// shared memory, which can be obtained from the other shared memory
    /// structs.
    pub fn get_str(&self, id: usize) -> Option<&str> {
        match self {
            FtlStrings::Production(strings) => Self::get_str_prod(strings, id),
            FtlStrings::Test(strings) => {
                if id == 0 {
                    Some("")
                } else {
                    strings.get(&id).map(|string| string.as_str())
                }
            }
        }
    }

    /// This function is used for `FtlStrings::Production`. It checks to see
    /// if the string exists, and then creates a `CStr` from a pointer. It
    /// is assumed that the string has a null terminator. Then the `CStr` is
    /// converted into `&str`. If the conversion fails, `None` is returned.
    fn get_str_prod(strings: &[libc::c_char], id: usize) -> Option<&str> {
        if strings.get(id).is_some() {
            unsafe { CStr::from_ptr(&strings[id]) }.to_str().ok()
        } else {
            None
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

        assert_eq!(strings.get_str(0), Some(""));
        assert_eq!(strings.get_str(1), Some("test"));
    }

    #[test]
    fn get_str_prod() {
        let strings: Vec<libc::c_char> = ['\0', 't', 'e', 's', 't', '\0']
            .iter()
            .map(|&c| c as libc::c_char)
            .collect();

        assert_eq!(FtlStrings::get_str_prod(&strings, 0), Some(""));
        assert_eq!(FtlStrings::get_str_prod(&strings, 1), Some("test"));
        assert_eq!(FtlStrings::get_str_prod(&strings, 6), None);
    }
}
