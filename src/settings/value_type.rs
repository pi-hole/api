// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Setting Value Types
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use regex::Regex;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// Categories of allowable values, shared across settings files
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueType {
    Boolean,
    ConditionalForwardingReverse,
    Decimal,
    Domain,
    Integer,
    Interface,
    Ipv4,
    Ipv4Mask,
    Ipv6,
    Filename,
    PortNumber,
    YesNo,
    WebPassword,
    String(&'static [&'static str])
}

impl ValueType {
    /// Validate submitted values for each category of settings entry value.
    ///
    /// Note: values are validated for format, not correctness.
    /// e.g. 0.1.2.3 is a valid IPV4, but may not be a valid upstream DNS
    pub fn is_valid(&self, value: &str) -> bool {
        match *self {
            ValueType::Boolean => {
                // True, False or null
                match value {
                    "true" | "false" | "" => true,
                    _ => false
                }
            }
            ValueType::ConditionalForwardingReverse => {
                // Specific reverse domain
                let reverse = Regex::new(
                    r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}([a-zA-Z0-9\-\.])+$"
                ).unwrap();
                reverse.is_match(value)
            }
            ValueType::Decimal => {
                // Numberic, at least one leading digit, optional decimal point and trailing
                // digits.
                let decimal = Regex::new(r"^(\d)+(\.)?(\d)*$").unwrap();
                decimal.is_match(value)
            }
            ValueType::Domain => {
                // Single word, hyphens allowed
                if value.is_empty() {
                    return true;
                };
                let domain =
                    Regex::new(r"^([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])$").unwrap();
                domain.is_match(value)
            }
            ValueType::Integer => {
                // Numeric - any number of digits
                let intnum = Regex::new(r"^(\d)+$").unwrap();
                intnum.is_match(value)
            }
            ValueType::Interface => {
                // Single alphanumeric word
                let domain =
                    Regex::new(r"^([a-zA-Z]|[a-zA-Z0-9][a-zA-Z0-9]*[a-zA-Z0-9])$").unwrap();
                domain.is_match(value)
            }
            ValueType::Ipv4 => {
                // Ipv4 4 octets, or null
                if value.is_empty() {
                    return true;
                };
                Ipv4Addr::from_str(value).is_ok()
            }
            ValueType::Ipv4Mask => {
                // IPv4 - 4 octets, with mask
                let ipv4 = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)/[0-9]+$").unwrap();
                ipv4.is_match(value)
            }
            ValueType::Ipv6 => {
                // IPv6 addresses, or null
                if value.is_empty() {
                    return true;
                };
                Ipv6Addr::from_str(value).is_ok()
            }
            ValueType::Filename => {
                // Full path filename, or null
                if value.is_empty() {
                    return true;
                }
                let filename = Regex::new(r"^(/(\S)+)+$").unwrap();
                filename.is_match(value)
            }
            ValueType::PortNumber => {
                // Port number, 0 - 65535
                let port = Regex::new(r"^((6553[0-5])|(655[0-2][0-9])|(65[0-4][0-9]{2})|(6[0-4][0-9]{3})|([1-5][0-9]{4})|([0-5]{0,5})|([0-9]{1,4}))$").unwrap();
                port.is_match(value)
            }
            ValueType::YesNo => {
                // Yes or no will do
                match value {
                    "yes" | "no" => true,
                    _ => false
                }
            }
            ValueType::WebPassword => {
                // Webpassword is a valid key, but altering it is disallowed
                false
            }
            ValueType::String(strings) => strings.contains(&value)
        }
    }
}
