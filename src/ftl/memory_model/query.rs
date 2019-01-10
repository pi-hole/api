// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Shared Memory Query Structure
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::ftl::FtlQueryType;
use libc;
use rocket::{http::RawStr, request::FromFormValue};

#[cfg(test)]
use crate::ftl::memory_model::MAGIC_BYTE;

/// The query struct stored in shared memory
#[repr(C)]
#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Copy, Clone)]
pub struct FtlQuery {
    magic: libc::c_uchar,
    pub timestamp: libc::time_t,
    pub time_index: libc::c_int,
    pub query_type: FtlQueryType,
    pub status: FtlQueryStatus,
    pub domain_id: libc::c_int,
    pub client_id: libc::c_int,
    pub upstream_id: libc::c_int,
    pub database_id: i64,
    pub id: libc::c_int,
    pub is_complete: bool,
    pub is_private: bool,
    /// Saved in units of 1/10 milliseconds (1 = 0.1ms, 2 = 0.2ms,
    /// 2500 = 250.0ms, etc.)
    pub response_time: libc::c_ulong,
    pub reply_type: FtlQueryReplyType,
    pub dnssec_type: FtlDnssecType,
    ad_bit: bool
}

impl FtlQuery {
    #[cfg(test)]
    pub fn new(
        id: isize,
        database_id: i64,
        timestamp: usize,
        time_index: usize,
        response_time: usize,
        domain_id: usize,
        client_id: usize,
        upstream_id: usize,
        query_type: FtlQueryType,
        status: FtlQueryStatus,
        reply_type: FtlQueryReplyType,
        dnssec_type: FtlDnssecType,
        is_complete: bool,
        is_private: bool
    ) -> FtlQuery {
        FtlQuery {
            magic: MAGIC_BYTE,
            id: id as libc::c_int,
            database_id,
            timestamp: timestamp as libc::time_t,
            time_index: time_index as libc::c_int,
            response_time: response_time as libc::c_ulong,
            domain_id: domain_id as libc::c_int,
            client_id: client_id as libc::c_int,
            upstream_id: upstream_id as libc::c_int,
            query_type,
            status,
            reply_type,
            dnssec_type,
            is_complete,
            is_private,
            ad_bit: false
        }
    }

    /// Check if the query was blocked
    pub fn is_blocked(&self) -> bool {
        match self.status {
            FtlQueryStatus::Gravity
            | FtlQueryStatus::Wildcard
            | FtlQueryStatus::Blacklist
            | FtlQueryStatus::ExternalBlock => true,
            _ => false
        }
    }
}

/// The statuses an FTL query can have
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FtlQueryStatus {
    Unknown,
    Gravity,
    Forward,
    Cache,
    Wildcard,
    Blacklist,
    ExternalBlock
}

impl<'v> FromFormValue<'v> for FtlQueryStatus {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        match form_value.parse::<u8>().map_err(|_| form_value)? {
            0 => Ok(FtlQueryStatus::Unknown),
            1 => Ok(FtlQueryStatus::Gravity),
            2 => Ok(FtlQueryStatus::Forward),
            3 => Ok(FtlQueryStatus::Cache),
            4 => Ok(FtlQueryStatus::Wildcard),
            5 => Ok(FtlQueryStatus::Blacklist),
            6 => Ok(FtlQueryStatus::ExternalBlock),
            _ => Err(form_value)
        }
    }
}

/// The reply types an FTL query can have
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FtlQueryReplyType {
    Unknown,
    NODATA,
    NXDOMAIN,
    CNAME,
    IP,
    DOMAIN,
    RRNAME
}

impl<'v> FromFormValue<'v> for FtlQueryReplyType {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        match form_value.parse::<u8>().map_err(|_| form_value)? {
            0 => Ok(FtlQueryReplyType::Unknown),
            1 => Ok(FtlQueryReplyType::NODATA),
            2 => Ok(FtlQueryReplyType::NXDOMAIN),
            3 => Ok(FtlQueryReplyType::CNAME),
            4 => Ok(FtlQueryReplyType::IP),
            5 => Ok(FtlQueryReplyType::DOMAIN),
            6 => Ok(FtlQueryReplyType::RRNAME),
            _ => Err(form_value)
        }
    }
}

/// The DNSSEC reply types an FTL query can have
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FtlDnssecType {
    Unspecified,
    Secure,
    Insecure,
    Bogus,
    Abandoned,
    Unknown
}

impl<'v> FromFormValue<'v> for FtlDnssecType {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        match form_value.parse::<u8>().map_err(|_| form_value)? {
            0 => Ok(FtlDnssecType::Unspecified),
            1 => Ok(FtlDnssecType::Secure),
            2 => Ok(FtlDnssecType::Insecure),
            3 => Ok(FtlDnssecType::Bogus),
            4 => Ok(FtlDnssecType::Abandoned),
            5 => Ok(FtlDnssecType::Unknown),
            _ => Err(form_value)
        }
    }
}
