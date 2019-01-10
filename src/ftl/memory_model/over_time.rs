// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Client Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use libc;

#[cfg(test)]
use crate::ftl::memory_model::MAGIC_BYTE;

#[repr(C)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone)]
pub struct FtlOverTime {
    magic: libc::c_uchar,
    pub timestamp: libc::time_t,
    pub total_queries: libc::c_int,
    pub blocked_queries: libc::c_int,
    pub cached_queries: libc::c_int,
    pub forwarded_queries: libc::c_int,
    query_types: [libc::c_int; 7]
}

impl FtlOverTime {
    #[cfg(test)]
    pub fn new(
        timestamp: usize,
        total_queries: usize,
        blocked_queries: usize,
        cached_queries: usize,
        forwarded_queries: usize,
        query_types: [libc::c_int; 7]
    ) -> FtlOverTime {
        FtlOverTime {
            magic: MAGIC_BYTE,
            timestamp: timestamp as libc::time_t,
            total_queries: total_queries as libc::c_int,
            blocked_queries: blocked_queries as libc::c_int,
            cached_queries: cached_queries as libc::c_int,
            forwarded_queries: forwarded_queries as libc::c_int,
            query_types
        }
    }
}
