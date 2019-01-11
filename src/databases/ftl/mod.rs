// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Support
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

mod schema;

pub use self::schema::*;

#[database("ftl_database")]
pub struct FtlDatabase(diesel::SqliteConnection);

#[allow(dead_code)]
pub enum FtlTableEntry {
    Version,
    LastTimestamp,
    FirstCounterTimestamp
}

#[allow(dead_code)]
pub enum CounterTableEntry {
    TotalQueries,
    BlockedQueries
}
