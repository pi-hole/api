// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Client Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::ftl::memory_model::{over_time::OVERTIME_SLOTS, strings::FtlStrings};
use libc;
use std::hash::{Hash, Hasher};

#[cfg(test)]
use crate::ftl::memory_model::MAGIC_BYTE;
#[cfg(test)]
use std::fmt::{
    self, {Debug, Formatter}
};

/// Represents an FTL client in API responses
#[derive(Serialize)]
pub struct ClientReply {
    pub name: String,
    pub ip: String
}

/// The client struct stored in shared memory.
///
/// Many traits, such as Debug and PartialEq, have to be manually implemented
/// because arrays longer than 32 do not implement these traits, which means
/// structs using them can not automatically derive the traits. For arrays of
/// any length to have these traits automatically implemented, the const
/// generics feature is required. That feature is still WIP:
/// https://github.com/rust-lang/rfcs/blob/master/text/2000-const-generics.md
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FtlClient {
    magic: libc::c_uchar,
    pub query_count: libc::c_int,
    pub blocked_count: libc::c_int,
    ip_str_id: libc::c_ulonglong,
    name_str_id: libc::c_ulonglong,
    is_name_unknown: bool,
    pub over_time: [libc::c_int; OVERTIME_SLOTS],
    last_query_time: libc::time_t,
    arp_query_count: libc::c_uint
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
            is_name_unknown: name_str_id.is_none(),
            over_time: [0; OVERTIME_SLOTS],
            last_query_time: 0,
            arp_query_count: 0
        }
    }

    /// Set the client's overTime data. The data given will be right-padded with
    /// zeros up to the required capacity (OVERTIME_SLOTS)
    #[cfg(test)]
    pub fn with_over_time(mut self, over_time: Vec<libc::c_int>) -> Self {
        let mut over_time_array = [0; OVERTIME_SLOTS];

        for (i, item) in over_time.into_iter().enumerate() {
            over_time_array[i] = item;
        }

        self.over_time = over_time_array;
        self
    }

    /// Get the IP address of the client
    pub fn get_ip<'a>(&self, strings: &'a FtlStrings) -> &'a str {
        strings.get_str(self.ip_str_id as usize).unwrap_or_default()
    }

    /// Get the name of the client, or `None` if it hasn't been resolved or
    /// doesn't exist
    pub fn get_name<'a>(&self, strings: &'a FtlStrings) -> Option<&'a str> {
        if !self.is_name_unknown && self.name_str_id != 0 {
            strings.get_str(self.name_str_id as usize)
        } else {
            None
        }
    }

    /// Convert this FTL client into the reply format
    pub fn as_reply(&self, strings: &FtlStrings) -> ClientReply {
        let name = self.get_name(&strings).unwrap_or_default();
        let ip = self.get_ip(&strings);

        ClientReply {
            name: name.to_owned(),
            ip: ip.to_owned()
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
            is_name_unknown: true,
            over_time: [0; OVERTIME_SLOTS],
            last_query_time: 0,
            arp_query_count: 0
        }
    }
}

impl PartialEq for FtlClient {
    fn eq(&self, other: &FtlClient) -> bool {
        self.magic == other.magic
            && self.query_count == other.query_count
            && self.blocked_count == other.blocked_count
            && self.ip_str_id == other.ip_str_id
            && self.name_str_id == other.name_str_id
            && self.is_name_unknown == other.is_name_unknown
            && self.over_time[..] == other.over_time[..]
            && self.last_query_time == other.last_query_time
            && self.arp_query_count == other.arp_query_count
    }
}

impl Eq for FtlClient {}

impl Hash for FtlClient {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.magic.hash(state);
        self.query_count.hash(state);
        self.blocked_count.hash(state);
        self.ip_str_id.hash(state);
        self.name_str_id.hash(state);
        self.is_name_unknown.hash(state);
        self.over_time.len().hash(state);
        Hash::hash_slice(&self.over_time, state);
        self.last_query_time.hash(state);
        self.arp_query_count.hash(state);
    }
}

#[cfg(test)]
impl Debug for FtlClient {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("FtlClient")
            .field("magic", &self.magic)
            .field("query_count", &self.query_count)
            .field("blocked_count", &self.blocked_count)
            .field("ip_str_id", &self.ip_str_id)
            .field("name_str_id", &self.name_str_id)
            .field("is_name_unknown", &self.is_name_unknown)
            .field("over_time", &format!("{:?}", self.over_time.to_vec()))
            .field("last_query_time", &self.last_query_time)
            .field("arp_query_count", &self.arp_query_count)
            .finish()
    }
}
