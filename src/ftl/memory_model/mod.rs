// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Shared Memory Data Types
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

#[cfg(test)]
use libc;

/// Used by FTL to check memory integrity in various structs
#[cfg(test)]
pub const MAGIC_BYTE: libc::c_uchar = 0x57;

mod client;
mod counters;
mod domain;
mod lock;
mod over_time;
mod query;
mod settings;
mod strings;
mod upstream;

pub use self::{
    client::FtlClient,
    counters::{FtlCounters, FtlQueryType},
    domain::{FtlDomain, FtlRegexMatch},
    lock::FtlLock,
    over_time::FtlOverTime,
    query::{FtlDnssecType, FtlQuery, FtlQueryReplyType, FtlQueryStatus},
    settings::FtlSettings,
    strings::FtlStrings,
    upstream::FtlUpstream
};
