// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Client Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use ftl::memory_model::strings::FtlStrings;
use libc;

#[cfg(test)]
use ftl::memory_model::MAGIC_BYTE;

/// The client struct stored in shared memory.
#[repr(C)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FtlClient {
    magic: libc::c_uchar,
    pub query_count: libc::c_int,
    pub blocked_count: libc::c_int,
    ip_str_id: libc::c_ulonglong,
    name_str_id: libc::c_ulonglong,
    is_name_unknown: bool
}

impl FtlClient {
    #[cfg(test)]
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

    /// Get the IP address of the client
    pub fn get_ip<'a>(&self, strings: &'a FtlStrings) -> &'a str {
        strings.get_str(self.ip_str_id as usize).unwrap_or_default()
    }

    /// Get the name of the client, or `None` if it hasn't been resolved or
    /// doesn't exist
    pub fn get_name<'a>(&self, strings: &'a FtlStrings) -> Option<&'a str> {
        if !self.is_name_unknown && self.name_str_id != 0 {
            Some(
                strings
                    .get_str(self.name_str_id as usize)
                    .unwrap_or_default()
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
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
