// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Counters Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use libc;

/// The FTL counters stored in shared memory
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct FtlCounters {
    pub total_queries: libc::c_int,
    pub blocked_queries: libc::c_int,
    pub cached_queries: libc::c_int,
    pub unknown_queries: libc::c_int,
    pub total_upstreams: libc::c_int,
    pub total_clients: libc::c_int,
    pub total_domains: libc::c_int,
    pub query_capacity: libc::c_int,
    pub upstream_capacity: libc::c_int,
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
#[derive(Copy, Clone, Debug)]
pub enum FtlQueryType {
    A,
    AAAA,
    ANY,
    SRV,
    SOA,
    PTR,
    TXT
}

impl FtlQueryType {
    /// A list of all `FtlQueryType` variants. There is no built in way to get
    /// a list of enum variants.
    pub fn variants() -> &'static [FtlQueryType] {
        &[
            FtlQueryType::A,
            FtlQueryType::AAAA,
            FtlQueryType::ANY,
            FtlQueryType::SRV,
            FtlQueryType::SOA,
            FtlQueryType::PTR,
            FtlQueryType::TXT
        ]
    }
}
