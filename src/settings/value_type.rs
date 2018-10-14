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
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum ValueType {
    Boolean,
    ConditionalForwardingReverse,
    Decimal,
    Domain,
    Filename,
    Integer,
    Interface,
    Ipv4,
    IPv4OptionalPort,
    Ipv4Mask,
    Ipv6,
    Path,
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
                file.file_name().is_some() && !value.ends_with("/")
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
            ValueType::IPv4OptionalPort => {
                // Valid, in allowable range, with optional port
                // (4 octets, with port from 0 to 65535, colon delimited), or null
                if value.is_empty() || is_ipv4_valid(value) {
                    return true;
                }
                if !value.contains(":") {
                    return false;
                }
                // check if port is specified
                let (ip, portnumber) = value.split_at(value.rfind(":").unwrap_or_default());
                if let Some(port) = portnumber.replace(":", "").parse::<usize>().ok() {
                    is_ipv4_valid(ip) && port <= 65535
                } else {
                    false
                }
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
            ValueType::Path => {
                // Valid full Path, or null
                if value.is_empty() {
                    return true;
                }

                // Test if a path and filename have been specified
                let path = Path::new(value);
                path.file_name().is_some() && path.is_absolute() && !value.ends_with("/")
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

#[cfg(test)]
mod tests {
    use super::{get_if_addrs, is_ipv4_valid, ValueType};

    #[test]
    fn test_value_type_valid() {
        let available_interface = get_if_addrs()
            .ok()
            .and_then(|interfaces| interfaces.into_iter().next())
            .map(|interface| interface.name)
            .unwrap_or_else(|| "lo".to_owned());

        let tests = vec![
            (ValueType::Boolean, "false", true),
            (
                ValueType::ConditionalForwardingReverse,
                "1.168.192.in-addr.arpa",
                true
            ),
            (ValueType::Decimal, "3.14", true),
            (ValueType::Domain, "domain", true),
            (ValueType::Filename, "c3po", true),
            (ValueType::Integer, "8675309", true),
            (ValueType::Interface, &available_interface, true),
            (ValueType::Ipv4, "192.168.2.9", true),
            (ValueType::IPv4OptionalPort, "192.168.4.5:80", true),
            (ValueType::IPv4OptionalPort, "192.168.3.3", true),
            (ValueType::Ipv4Mask, "192.168.0.3/24", true),
            (
                ValueType::Ipv6,
                "f7c4:12f8:4f5a:8454:5241:cf80:d61c:3e2c",
                true
            ),
            (ValueType::Path, "/tmp/directory/file.ext", true),
            (ValueType::PortNumber, "9000", true),
            (ValueType::YesNo, "yes", true),
            (ValueType::String(&["boxed", ""]), "boxed", true),
        ];

        for (setting, value, result) in tests {
            assert_eq!(
                setting.is_valid(value),
                result,
                "{:?}.is_valid({:?}) == {}",
                setting,
                value,
                result
            );
        }
    }

    #[test]
    fn test_value_type_invalid() {
        let tests = vec![
            (ValueType::Boolean, "yes", false),
            (
                ValueType::ConditionalForwardingReverse,
                "www.pi-hole.net",
                false
            ),
            (ValueType::Decimal, "3/4", false),
            (ValueType::Decimal, "3.14.15.26", false),
            (ValueType::Domain, "D0#A!N", false),
            (ValueType::Filename, "c3p0/", false),
            (ValueType::Integer, "9.9", false),
            (ValueType::Integer, "10m3", false),
            (ValueType::Interface, "/dev/net/ev9d9", false),
            (ValueType::Ipv4, "192.168.0.3/24", false),
            (ValueType::Ipv4, "192.168.0.2:53", false),
            (ValueType::IPv4OptionalPort, "192.168.4.5 port 1000", false),
            (ValueType::IPv4OptionalPort, "192.168.6.8:arst", false),
            (ValueType::Ipv4Mask, "192.168.2.9", false),
            (ValueType::Ipv4Mask, "192.168.1.1/qwfp", false),
            (ValueType::Ipv6, "192.168.0.3", false),
            (ValueType::Path, "~/tmp/directory/file.ext", false),
            (ValueType::PortNumber, "65536", false),
            (ValueType::YesNo, "true", false),
            (ValueType::String(&["boxed", ""]), "lan", false),
        ];

        for (setting, value, result) in tests {
            assert_eq!(
                setting.is_valid(value),
                result,
                "{:?}.is_valid({:?}) == {}",
                setting,
                value,
                result
            );
        }
    }

    #[test]
    fn test_ipv4_ranges() {
        let tests = vec![
            // Valid ip
            ("192.168.0.1", true),
            // Test single ip from each invalid range
            ("255.255.255.255", false), // broadcast
            ("192.0.2.1", false),       // documentation
            ("169.254.0.2", false),     // link-local
            ("239.255.255.255", false), // multicast
            ("0.0.0.0", false),         // unspecified
        ];

        for (value, result) in tests {
            assert_eq!(is_ipv4_valid(value), result);
        }
    }
}
