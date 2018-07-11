// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Setting Value Types
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use get_if_addrs::get_if_addrs;
use regex::Regex;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::str::FromStr;

/// Categories of allowable values, shared across settings files
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueType {
    Boolean,
    ConditionalForwardingReverse,
    Decimal,
    Domain,
    Filename,
    Integer,
    Interface,
    Ipv4,
    Ipv4Mask,
    Ipv6,
    Pathname,
    PortNumber,
    YesNo,
    WebPassword,
    String(&'static [&'static str])
}

impl ValueType {
    /// Check if the value is valid for this entry
    ///
    /// Note: values are validated for format, not correctness.
    /// e.g. 0.1.2.3 is a valid IPV4, but may not be a valid upstream DNS
    pub fn is_valid(&self, value: &str) -> bool {
        match *self {
            ValueType::Boolean => match value {
                "true" | "false" | "" => true,
                _ => false
            },
            ValueType::ConditionalForwardingReverse => {
                // Specific reverse domain
                let reverse_re = Regex::new(
                    r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}([a-zA-Z0-9\-\.])+$"
                ).unwrap();
                reverse_re.is_match(value)
            }
            ValueType::Decimal => {
                // Numeric, at least one leading digit, optional decimal point and trailing
                // digits.
                let decimal_re = Regex::new(r"^(\d)+(\.)?(\d)*$").unwrap();
                decimal_re.is_match(value)
            }
            ValueType::Domain => {
                // Single word, hyphens allowed
                if value.is_empty() {
                    return true;
                }

                let domain_re =
                    Regex::new(r"^([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])$").unwrap();
                domain_re.is_match(value)
            }
            ValueType::Filename => {
                // Valid file, or null
                if value.is_empty() {
                    return true;
                }

                let file = Path::new(value);
                file.file_name().is_some()
            }
            ValueType::Integer => {
                // At least one digit
                let numeric_re = Regex::new(r"^(\d)+$").unwrap();
                numeric_re.is_match(value)
            }
            ValueType::Interface => {
                // Interface present on system
                get_if_addrs()
                    .unwrap_or_default()
                    .iter()
                    .any(|interface| interface.name == value)
            }
            ValueType::Ipv4 => {
                // Valid and in allowable range
                // (4 octets, or null)
                // Test if valid address falls within permitted ranges
                value.is_empty() || is_ipv4_valid(value)
            }
            ValueType::Ipv4Mask => {
                // Valid, in allowable range, and with mask
                // (4 octets, with mask)
                if !value.contains("/") {
                    return false;
                }

                let (ip, mask) = value.split_at(value.rfind("/").unwrap_or_default());
                let numeric_re = Regex::new(r"^(\d)+$").unwrap();

                numeric_re.is_match(&mask[1..]) && is_ipv4_valid(ip)
            }
            ValueType::Ipv6 => {
                // IPv6 addresses in allowable range, or null
                if value.is_empty() {
                    return true;
                }

                match Ipv6Addr::from_str(value) {
                    Ok(ipv6) => {
                        // Prohibited address ranges: Multicast & Unspecified
                        // (all others permitted)
                        !ipv6.is_multicast() && !ipv6.is_unspecified()
                    }
                    Err(_) => return false
                }
            }
            ValueType::Pathname => {
                // Valid full pathname, or null
                if value.is_empty() {
                    return true;
                }

                // Test if a path and filename have been specified
                let path = Path::new(value);
                path.file_name().is_some() && path.is_absolute() && !path.ends_with("/")
            }
            ValueType::PortNumber => {
                // Number from 0 to 65535
                if let Some(port) = value.parse::<usize>().ok() {
                    port <= 65535
                } else {
                    false
                }
            }
            ValueType::YesNo => match value {
                "yes" | "no" => true,
                _ => false
            },
            ValueType::WebPassword => {
                // Web password is a valid key, but altering it is disallowed
                false
            }
            ValueType::String(strings) => strings.contains(&value)
        }
    }
}

/// IPv4 - Check that specified address is valid
fn is_ipv4_valid(value: &str) -> bool {
    match Ipv4Addr::from_str(value) {
        Ok(ipv4) => {
            // Prohibited address ranges
            // Broadcast, Documentation, Link-local, Multicast & Unspecified
            // (all others permitted)
            !ipv4.is_broadcast()
                && !ipv4.is_documentation()
                && !ipv4.is_link_local()
                && !ipv4.is_multicast()
                && !ipv4.is_unspecified()
        }
        Err(_) => return false
    }
}
