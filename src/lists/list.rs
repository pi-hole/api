// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// The List Enumeration
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::settings::ValueType;

/// Represents the various Pi-hole domain lists
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum List {
    White,
    Black,
    Regex
}

impl List {
    /// Check if the list accepts the domain as valid
    pub fn accepts(self, domain: &str) -> bool {
        match self {
            List::Regex => ValueType::Regex.is_valid(domain),
            // Allow hostnames to be white/blacklist-ed
            _ => ValueType::Hostname.is_valid(domain)
        }
    }
}
