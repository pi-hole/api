// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Models
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::ftl::{FtlDnssecType, FtlQueryReplyType};
use rocket_contrib::json::JsonValue;

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

#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Queryable)]
pub struct FtlDbQuery {
    pub id: Option<i32>,
    pub timestamp: i32,
    pub query_type: i32,
    pub status: i32,
    pub domain: String,
    pub client: String,
    pub upstream: Option<String>
}

impl Into<JsonValue> for FtlDbQuery {
    fn into(self) -> JsonValue {
        json!({
            "timestamp": self.timestamp,
            "type": self.query_type as u8,
            "status": self.status as u8,
            "domain": self.domain,
            "client": self.client,
            "dnssec": FtlDnssecType::Unknown as u8,
            "reply": FtlQueryReplyType::Unknown as u8,
            "response_time": 0
        })
    }
}
