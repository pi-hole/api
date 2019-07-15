// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Models
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::foreign_key_connection::SqliteFKConnection,
    ftl::{FtlDnssecType, FtlQueryReplyType},
    routes::stats::QueryReply
};

#[database("ftl_database")]
pub struct FtlDatabase(SqliteFKConnection);

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
    pub id: i32,
    pub timestamp: i32,
    pub query_type: i32,
    pub status: i32,
    pub domain: String,
    pub client: String,
    pub upstream: Option<String>
}

impl Into<QueryReply> for FtlDbQuery {
    fn into(self) -> QueryReply {
        QueryReply {
            timestamp: self.timestamp as u64,
            r#type: self.query_type as u8,
            status: self.status as u8,
            domain: self.domain,
            client: self.client,
            dnssec: FtlDnssecType::Unknown as u8,
            reply: FtlQueryReplyType::Unknown as u8,
            response_time: 0
        }
    }
}
