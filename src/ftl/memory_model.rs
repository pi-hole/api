// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Structs
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use libc;
use shmem::Array;
use std::collections::HashMap;
use std::ffi::CStr;
use std::str::FromStr;
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
    pub fn new(
        query_count: usize,
        blocked_count: usize,
        ip_str_id: usize,
        name_str_id: Option<usize>
    ) -> FtlClient {
        FtlClient {
            magic: MAGIC_BYTE,
            query_count: query_count as libc::c_int,
            blocked_count: blocked_count as libc::c_int,
            ip_str_id: ip_str_id as libc::c_ulonglong,
            name_str_id: name_str_id.unwrap_or_default() as libc::c_ulonglong,
            is_name_unknown: name_str_id.is_none()
        }
    }

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

/// The FTL counters stored in shared memory
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct FtlCounters {
    pub total_queries: libc::c_int,
    pub blocked_queries: libc::c_int,
    pub cached_queries: libc::c_int,
    pub unknown_queries: libc::c_int,
    pub forward_destinations: libc::c_int,
    pub total_clients: libc::c_int,
    pub total_domains: libc::c_int,
    pub query_capacity: libc::c_int,
    pub forward_destination_capacity: libc::c_int,
    pub client_capacity: libc::c_int,
    pub domain_capacity: libc::c_int,
    pub over_time_capacity: libc::c_int,
    pub gravity_size: libc::c_int,
    pub gravity_conf: libc::c_int,
    pub over_time_size: libc::c_int,
    pub query_type_counters: [libc::c_int; 7],
    pub forwarded_queries: libc::c_int,
    pub reply_count_nodata: libc::c_int,
    pub reply_count_nxdomain: libc::c_int,
    pub reply_count_cname: libc::c_int,
    pub reply_count_ip: libc::c_int,
    pub reply_count_domain: libc::c_int
}

impl FtlCounters {
    pub fn query_type(&self, query_type: FtlQueryType) -> usize {
        self.query_type_counters[query_type as usize] as usize
    }
}

/// The query types stored by FTL. Use this enum for [`FtlCounters::query_type`]
///
/// [`FtlCounters::query_type`]: struct.FtlCounters.html#method.query_type
pub enum FtlQueryType {
    A = 1,
    AAAA,
    ANY,
    SRV,
    SOA,
    PTR,
    TXT
}

/// The privacy levels used by FTL
#[derive(PartialOrd, PartialEq)]
pub enum FtlPrivacyLevel {
    ShowAll,
    HideDomains,
    HideDomainsAndClients,
    Maximum,
    NoStats
}

impl FromStr for FtlPrivacyLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "0" => Ok(FtlPrivacyLevel::ShowAll),
            "1" => Ok(FtlPrivacyLevel::HideDomains),
            "2" => Ok(FtlPrivacyLevel::HideDomainsAndClients),
            "3" => Ok(FtlPrivacyLevel::Maximum),
            "4" => Ok(FtlPrivacyLevel::NoStats),
            _ => Err(Error::from(ErrorKind::InvalidSettingValue))
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
