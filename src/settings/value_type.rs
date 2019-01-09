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
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
    str::FromStr
};

/// Categories of allowable values, shared across settings files
#[cfg_attr(test, derive(Debug))]
pub enum ValueType {
    Boolean,
    /// A comma separated array of strings which match at least one of the
    /// specified value types
    Array(&'static [ValueType]),
    ConditionalForwardingReverse,
    Decimal,
    Domain,
    Filename,
    Hostname,
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
            ValueType::Array(value_types) => value.split(",").all(|value| {
                value_types
                    .iter()
                    .any(|value_type| value_type.is_valid(value))
            }),
            ValueType::Boolean => match value {
                "true" | "false" => true,
                _ => false
            },
            ValueType::ConditionalForwardingReverse => {
                // Specific reverse domain
                let reverse_re = Regex::new(
                    r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}([a-zA-Z0-9\-\.])+$"
                )
                .unwrap();
                reverse_re.is_match(value)
            }
            ValueType::Decimal => {
                // Numeric, at least one leading digit, optional decimal point and trailing
                // digits.
                let decimal_re = Regex::new(r"^(\d)+(\.)?(\d)*$").unwrap();
                decimal_re.is_match(value)
            }
            ValueType::Domain => {
                // Like a hostname, but must be fully qualified
                let split: Vec<&str> = value.split('.').collect();

                // Must have at least two segments/labels of one or more characters
                if split.len() < 2 || split.iter().any(|label| label.is_empty()) {
                    return false;
                }

                ValueType::Hostname.is_valid(value)
            }
            ValueType::Filename => {
                Path::new(value).file_name().is_some()
                    && !value.ends_with("/")
                    && !value.ends_with("/.")
            }
            ValueType::Hostname => {
                // A hostname must not exceed 253 characters
                if value.len() > 253 {
                    return false;
                }

                // The segments/labels of a hostname must not exceed 63 characters each
                if value.split(".").any(|label| label.len() > 63) {
                    return false;
                }

                // A hostname can not be all numbers and periods
                if value.split(".").all(|label| label.parse::<usize>().is_ok()) {
                    return false;
                }

                let hostname_re = Regex::new(
                    r"^([a-zA-Z0-9]+(-[a-zA-Z0-9]+)*)+(\.([a-zA-Z0-9]+(-[a-zA-Z0-9]+)*))*$"
                )
                .unwrap();
                hostname_re.is_match(value)
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
                // (4 octets)
                // Test if valid address falls within permitted ranges
                is_ipv4_valid(value)
            }
            ValueType::IPv4OptionalPort => {
                // Valid, in allowable range, with optional port
                // (4 octets, with port from 0 to 65535, colon delimited)
                if is_ipv4_valid(value) {
                    return true;
                }

                if !value.contains(":") {
                    return false;
                }

                // Check if port is specified
                let (ip, port) = value.split_at(value.rfind(":").unwrap());
                if let Some(port) = port.replace(":", "").parse::<usize>().ok() {
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

                let (ip, mask) = value.split_at(value.rfind("/").unwrap());
                ValueType::Integer.is_valid(&mask.replace("/", "")) && is_ipv4_valid(ip)
            }
            ValueType::Ipv6 => {
                if let Ok(ipv6) = Ipv6Addr::from_str(value) {
                    // Prohibited address ranges: Multicast & Unspecified
                    // (all others permitted)
                    !ipv6.is_multicast() && !ipv6.is_unspecified()
                } else {
                    false
                }
            }
            ValueType::Path => {
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
                ValueType::Array(&[ValueType::Hostname, ValueType::Ipv4]),
                "pi.hole,127.0.0.1",
                true
            ),
            (
                ValueType::ConditionalForwardingReverse,
                "1.168.192.in-addr.arpa",
                true
            ),
            (ValueType::Decimal, "3.14", true),
            (ValueType::Domain, "domain.com", true),
            (ValueType::Filename, "c3po", true),
            (ValueType::Hostname, "localhost", true),
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
                ValueType::Array(&[ValueType::Hostname, ValueType::Ipv4]),
                "123, $test,",
                false
            ),
            (
                ValueType::Array(&[ValueType::Hostname, ValueType::Ipv4]),
                "123,",
                false
            ),
            (
                ValueType::ConditionalForwardingReverse,
                "www.pi-hole.net",
                false
            ),
            (ValueType::Decimal, "3/4", false),
            (ValueType::Decimal, "3.14.15.26", false),
            (ValueType::Domain, "D0#A!N", false),
            (ValueType::Filename, "c3p0/", false),
            (ValueType::Hostname, ".localhost", false),
            (ValueType::Hostname, "localhost.", false),
            (ValueType::Hostname, "127.0.0.1", false),
            (ValueType::Hostname, "my.ho$t.name", false),
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
