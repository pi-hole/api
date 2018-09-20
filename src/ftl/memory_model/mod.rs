// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
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
const MAGIC_BYTE: libc::c_uchar = 0x57;

mod client;
mod counters;
mod domain;
mod over_time;
mod query;
mod strings;
mod upstream;

pub use self::client::FtlClient;
pub use self::counters::{FtlCounters, FtlQueryType};
pub use self::domain::{FtlDomain, FtlRegexMatch};
pub use self::over_time::FtlOverTime;
pub use self::query::{FtlDnssecType, FtlQuery, FtlQueryReplyType, FtlQueryStatus};
pub use self::strings::FtlStrings;
pub use self::upstream::FtlUpstream;
