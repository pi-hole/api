// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
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
    /// A value which matches at least one of the specified value types
    Any(&'static [ValueType]),
    /// A comma separated array of strings which match at least one of the
    /// specified value types
    Array(&'static [ValueType]),
    Boolean,
    Decimal,
    Domain,
    #[allow(dead_code)]
    Filename,
    Hostname,
    Integer,
    Interface,
    IPv4,
    IPv4OptionalPort,
    IPv4Mask,
    IPv4CIDR,
    IPv6,
    IPv6OptionalPort,
    IPv6CIDR,
    Path,
    PortNumber,
    Regex,
    YesNo,
    WebPassword,
    String(&'static [&'static str]),
    LanguageCode
}

impl ValueType {
    /// Check if the value is valid for this entry
    ///
    /// Note: values are validated for format, not correctness.
    /// e.g. 0.1.2.3 is a valid IPV4, but may not be a valid upstream DNS
    pub fn is_valid(&self, value: &str) -> bool {
        match self {
            ValueType::Any(value_types) => value_types
                .iter()
                .any(|value_type| value_type.is_valid(value)),
            ValueType::Array(value_types) => value.split(',').all(|value| {
                value_types
                    .iter()
                    .any(|value_type| value_type.is_valid(value))
            }),
            ValueType::Boolean => match value {
                "true" | "false" => true,
                _ => false
            },
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
                    && !value.ends_with('/')
                    && !value.ends_with("/.")
            }
            ValueType::Hostname => {
                // A hostname must not exceed 253 characters
                if value.len() > 253 {
                    return false;
                }

                // The segments/labels of a hostname must not exceed 63 characters each
                if value.split('.').any(|label| label.len() > 63) {
                    return false;
                }

                // A hostname can not be all numbers and periods
                if value.split('.').all(|label| label.parse::<usize>().is_ok()) {
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
            ValueType::IPv4 => {
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

                if !value.contains(':') {
                    return false;
                }

                // Check if port is specified
                let (ip, port) = value.split_at(value.rfind(':').unwrap());
                if let Ok(port) = port.replace(":", "").parse::<usize>() {
                    is_ipv4_valid(ip) && port <= 65535
                } else {
                    false
                }
            }
            ValueType::IPv4Mask => {
                // Valid, in allowable range, and with mask
                // (4 octets, with mask)
                if !value.contains('/') {
                    return false;
                }

                let (ip, mask) = value.split_at(value.rfind('/').unwrap());
                ValueType::Integer.is_valid(&mask.replace("/", "")) && is_ipv4_valid(ip)
            }
            ValueType::IPv4CIDR => ValueType::String(&["8", "16", "24", "32"]).is_valid(value),
            ValueType::IPv6 => is_ipv6_valid(value),
            ValueType::IPv6OptionalPort => get_ipv6_address_and_port(value).is_some(),
            ValueType::IPv6CIDR => {
                // The CIDR must be a positive number
                let cidr: usize = match value.parse() {
                    Ok(cidr) => cidr,
                    Err(_) => return false
                };

                cidr > 0 && cidr <= 128 && cidr % 4 == 0
            }
            ValueType::Path => {
                // Test if a path and filename have been specified
                let path = Path::new(value);
                path.file_name().is_some() && path.is_absolute() && !value.ends_with('/')
            }
            ValueType::PortNumber => {
                // Number from 0 to 65535
                if let Ok(port) = value.parse::<usize>() {
                    port <= 65535
                } else {
                    false
                }
            }
            ValueType::Regex => Regex::new(value).is_ok(),
            ValueType::YesNo => match value {
                "yes" | "no" => true,
                _ => false
            },
            ValueType::WebPassword => {
                // Web password is a valid key, but altering it is disallowed
                false
            }
            ValueType::String(strings) => strings.contains(&value),
            ValueType::LanguageCode => Regex::new("^[a-zA-Z]+(-[a-zA-Z]+)*$")
                .unwrap()
                .is_match(value)
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
        Err(_) => false
    }
}

/// IPv6 - Check that the specified address is valid
fn is_ipv6_valid(value: &str) -> bool {
    match Ipv6Addr::from_str(value) {
        Ok(ipv6) => {
            // Prohibited address ranges: Multicast & Unspecified
            // (all others permitted)
            !ipv6.is_multicast() && !ipv6.is_unspecified()
        }
        Err(_) => false
    }
}

/// Get the address and port of an string representing an IPv6 address with or
/// without a port.
///
/// If the address is invalid, None is returned.
/// Otherwise, the returned tuple represents the address and optional port
pub fn get_ipv6_address_and_port(value: &str) -> Option<(&str, Option<usize>)> {
    // If this is a valid IPv6 address without a port, we can stop here
    if is_ipv6_valid(value) {
        return Some((value, None));
    }

    // Extract the address and port using Regex
    let ipv6_re = Regex::new(r"^\[([a-fA-F0-9:]+)]:(\d+)$").unwrap();

    // If the value does not match the regex, we return None via ?
    let captures = ipv6_re.captures(value)?;

    // Get the IPv6 address and port
    let address = captures.get(1).map(|m| m.as_str())?;
    let port: usize = captures[2].parse().ok()?;

    if is_ipv6_valid(address) && port <= 65535 {
        Some((address, Some(port)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{get_if_addrs, get_ipv6_address_and_port, is_ipv4_valid, ValueType};

    #[test]
    fn test_value_type_valid() {
        let available_interface = get_if_addrs()
            .ok()
            .and_then(|interfaces| interfaces.into_iter().next())
            .map(|interface| interface.name)
            .unwrap_or_else(|| "lo".to_owned());

        let tests = vec![
            (
                ValueType::Any(&[ValueType::Integer, ValueType::Boolean]),
                "1234"
            ),
            (
                ValueType::Any(&[ValueType::Integer, ValueType::Boolean]),
                "true"
            ),
            (
                ValueType::Array(&[ValueType::Hostname, ValueType::IPv4]),
                "pi.hole,127.0.0.1"
            ),
            (ValueType::Boolean, "false"),
            (ValueType::Decimal, "3.14"),
            (ValueType::Domain, "domain.com"),
            (ValueType::Filename, "c3po"),
            (ValueType::Hostname, "localhost"),
            (ValueType::Integer, "8675309"),
            (ValueType::Interface, &available_interface),
            (ValueType::IPv4, "192.168.2.9"),
            (ValueType::IPv4OptionalPort, "192.168.4.5:80"),
            (ValueType::IPv4OptionalPort, "192.168.3.3"),
            (ValueType::IPv4Mask, "192.168.0.3/24"),
            (ValueType::IPv4CIDR, "24"),
            (ValueType::IPv6, "f7c4:12f8:4f5a:8454:5241:cf80:d61c:3e2c"),
            (
                ValueType::IPv6OptionalPort,
                "f7c4:12f8:4f5a:8454:5241:cf80:d61c:3e2c"
            ),
            (ValueType::IPv6OptionalPort, "[1fff:0:a88:85a3::ac1f]:8001"),
            (ValueType::IPv6CIDR, "64"),
            (ValueType::Path, "/tmp/directory/file.ext"),
            (ValueType::PortNumber, "9000"),
            (ValueType::Regex, "^.*example$"),
            (ValueType::YesNo, "yes"),
            (ValueType::String(&["boxed", ""]), "boxed"),
        ];

        for (setting, value) in tests {
            let result = setting.is_valid(value);

            assert_eq!(
                result, true,
                "{:?}.is_valid({:?}) == {}",
                setting, value, result
            );
        }
    }

    #[test]
    fn test_value_type_invalid() {
        let tests = vec![
            (
                ValueType::Any(&[ValueType::Integer, ValueType::Boolean]),
                "3/4"
            ),
            (
                ValueType::Any(&[ValueType::Integer, ValueType::Boolean]),
                "192.168.1.1"
            ),
            (
                ValueType::Array(&[ValueType::Hostname, ValueType::IPv4]),
                "123, $test,"
            ),
            (
                ValueType::Array(&[ValueType::Hostname, ValueType::IPv4]),
                "123,"
            ),
            (ValueType::Boolean, "yes"),
            (ValueType::Decimal, "3/4"),
            (ValueType::Decimal, "3.14.15.26"),
            (ValueType::Domain, "D0#A!N"),
            (ValueType::Filename, "c3p0/"),
            (ValueType::Hostname, ".localhost"),
            (ValueType::Hostname, "localhost."),
            (ValueType::Hostname, "127.0.0.1"),
            (ValueType::Hostname, "my.ho$t.name"),
            (ValueType::Integer, "9.9"),
            (ValueType::Integer, "10m3"),
            (ValueType::Interface, "/dev/net/ev9d9"),
            (ValueType::IPv4, "192.168.0.3/24"),
            (ValueType::IPv4, "192.168.0.2:53"),
            (ValueType::IPv4OptionalPort, "192.168.4.5 port 1000"),
            (ValueType::IPv4OptionalPort, "192.168.6.8:arst"),
            (ValueType::IPv4Mask, "192.168.2.9"),
            (ValueType::IPv4Mask, "192.168.1.1/qwfp"),
            (ValueType::IPv4CIDR, "-1"),
            (ValueType::IPv4CIDR, "124"),
            (ValueType::IPv6, "192.168.0.3"),
            (ValueType::IPv6OptionalPort, "192.168.0.3"),
            (ValueType::IPv6OptionalPort, "1fff:0:a88:85a3::ac1f#8001"),
            (ValueType::IPv6CIDR, "-1"),
            (ValueType::IPv6CIDR, "23"),
            (ValueType::IPv6CIDR, "150"),
            (ValueType::Path, "~/tmp/directory/file.ext"),
            (ValueType::PortNumber, "65536"),
            (ValueType::Regex, "example\\"),
            (ValueType::YesNo, "true"),
            (ValueType::String(&["boxed", ""]), "lan"),
        ];

        for (setting, value) in tests {
            let result = setting.is_valid(value);

            assert_eq!(
                result, false,
                "{:?}.is_valid({:?}) == {}",
                setting, value, result
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

    /// `get_ipv6_address_and_port` returns a tuple without a port when given
    /// an IPv6 address with no port
    #[test]
    fn get_ipv6_info_no_port() {
        assert_eq!(
            get_ipv6_address_and_port("f7c4:12f8:4f5a:8454:5241:cf80:d61c:3e2c"),
            Some(("f7c4:12f8:4f5a:8454:5241:cf80:d61c:3e2c", None))
        );
    }

    /// `get_ipv6_address_and_port` returns a tuple with a port when given an
    /// IPv6 address with a port
    #[test]
    fn get_ipv6_info_with_port() {
        assert_eq!(
            get_ipv6_address_and_port("[1fff:0:a88:85a3::ac1f]:8001"),
            Some(("1fff:0:a88:85a3::ac1f", Some(8001)))
        );
    }
}
