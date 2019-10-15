// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Domain Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::ftl::FtlStrings;
use libc;

#[cfg(test)]
use crate::ftl::memory_model::MAGIC_BYTE;

/// The domain struct stored in shared memory
#[repr(C)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct FtlDomain {
    magic: libc::c_uchar,
    pub regex_match: FtlRegexMatch,
    domain_str_id: libc::size_t,
    pub query_count: libc::c_int,
    pub blocked_count: libc::c_int
}

impl FtlDomain {
    #[cfg(test)]
    pub fn new(
        total_count: usize,
        blocked_count: usize,
        domain_str_id: usize,
        regex_match: FtlRegexMatch
    ) -> FtlDomain {
        FtlDomain {
            magic: MAGIC_BYTE,
            query_count: total_count as libc::c_int,
            blocked_count: blocked_count as libc::c_int,
            domain_str_id: domain_str_id as libc::size_t,
            regex_match
        }
    }

    /// Get the domain name
    pub fn get_domain<'a>(&self, strings: &'a FtlStrings) -> &'a str {
        strings
            .get_str(self.domain_str_id as usize)
            .unwrap_or_default()
    }
}

#[cfg(test)]
impl Default for FtlDomain {
    fn default() -> Self {
        FtlDomain {
            magic: MAGIC_BYTE,
            query_count: 0,
            blocked_count: 0,
            domain_str_id: 0,
            regex_match: FtlRegexMatch::Unknown
        }
    }
}

/// The regex state a domain can hold. Unknown is the default state, before it
/// is checked when a query of the domain comes in.
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[allow(dead_code)]
pub enum FtlRegexMatch {
    Unknown,
    Blocked,
    NotBlocked
}
