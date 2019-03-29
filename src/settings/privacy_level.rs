// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Privacy Level Enum
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::util::{Error, ErrorKind};
use std::str::FromStr;

/// The privacy levels used by FTL
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum FtlPrivacyLevel {
    ShowAll,
    HideDomains,
    HideDomainsAndClients,
    Maximum,
    NoStats
}

impl FromStr for FtlPrivacyLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "0" => Ok(FtlPrivacyLevel::ShowAll),
            "1" => Ok(FtlPrivacyLevel::HideDomains),
            "2" => Ok(FtlPrivacyLevel::HideDomainsAndClients),
            "3" => Ok(FtlPrivacyLevel::Maximum),
            "4" => Ok(FtlPrivacyLevel::NoStats),
            _ => Err(Error::from(ErrorKind::InvalidSettingValue))
        }
    }
}
