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
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;
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
                // Numeric, at least one leading digit, optional decimal point and trailing
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
            ValueType::Filename => {
                // Valid file, or null
                if value.is_empty() {
                    return true;
                };
                // test if full pathname
                let filename = Regex::new(r"^(/(\S)+)+$").unwrap();
                if !filename.is_match(value) {
                    return false;
                };
                // test if directory exists, and filename has been specified
                let (directory, filename) = value.split_at(value.rfind("/").unwrap_or_default());
                if filename != "/" {
                    return Path::new(directory).exists();
                };
                false
            }
            ValueType::Integer => {
                // Numeric - any number of digits
                let intnum = Regex::new(r"^(\d)+$").unwrap();
                intnum.is_match(value)
            }
            ValueType::Interface => {
                // Interface - device listed in /proc/net/dev
                // (Single alphanumeric word)
                let domain =
                    Regex::new(r"^([a-zA-Z]|[a-zA-Z0-9][a-zA-Z0-9]*[a-zA-Z0-9])$").unwrap();
                if !domain.is_match(value) {
                    return false;
                };
                for device in get_net_devs().iter() {
                    if value == device {
                        return true;
                    }
                }
                false
            }
            ValueType::Ipv4 => {
                // Ipv4 - valid and in allowable range
                // (4 octets, or null)
                if value.is_empty() {
                    return true;
                };
                // Test if valid address falls within permitted ranges
                is_ipv4_allowable(value)
            }
            ValueType::Ipv4Mask => {
                // Ipv4 - in allowable range, with mask
                // (4 octets, with mask)
                let ipv4 = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)/[0-9]+$").unwrap();
                if !ipv4.is_match(value) {
                    return false;
                }
                let (ip, _mask) = value.split_at(value.find("/").unwrap_or_default());
                is_ipv4_allowable(ip)
            }
            ValueType::Ipv6 => {
                // IPv6 addresses in allowable range, or null
                if value.is_empty() {
                    return true;
                };
                match Ipv6Addr::from_str(value) {
                    Ok(ipv6) => {
                        // Prohibited address ranges - IETF RFC 4291
                        // Multicast
                        if ipv6.is_multicast() {
                            return false;
                        }
                        // Unspecified
                        if ipv6.is_unspecified() {
                            return false;
                        }
                        // Global addresses are permitted
                        // Loopback addresses are permitted
                        // Link-local adresses are permitted
                        //
                        return true;
                    }
                    Err(_) => return false
                }
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
                // Web password is a valid key, but altering it is disallowed
                false
            }
            ValueType::String(strings) => strings.contains(&value)
        }
    }
}

/// Interface - Check that specified interface matches one of the
/// current list of network interfaces (from /proc/net/dev)
fn get_net_devs() -> Vec<String> {
    // Get the current list of network interfaces
    let mut net_devs = Vec::new();
    // Parse output of /proc/net/dev - entries are terminated by a colon,
    // found at the start of a line but may have prepended spaces
    // eg: "\n    eth0: 000 001" "\nvirbr0: 000 001"
    // If unable to read device list, return null array.
    match File::open("/proc/net/dev") {
        Ok(f) => {
            let file = BufReader::new(f);
            let devlist = file
                .lines()
                .filter_map(|item| item.ok())
                .filter(|line| line.contains(':'));
            for line in devlist {
                let device = line[0..line.find(':').unwrap_or_default()]
                    .trim_left()
                    .to_string();
                net_devs.push(device);
            }
        }
        _ => {}
    };
    net_devs
}

/// IPv4 - Check that specified address is allowable
fn is_ipv4_allowable(value: &str) -> bool {
    match Ipv4Addr::from_str(value) {
        Ok(ipv4) => {
            // Prohibited address ranges
            // Broadcast - IETF RFC 919
            if ipv4.is_broadcast() {
                return false;
            }
            // Documentation - IETF 5737
            if ipv4.is_documentation() {
                return false;
            }
            // Link-local - IETF RFC 3927
            if ipv4.is_link_local() {
                return false;
            }
            // Multicast - IETF RFC 5771
            if ipv4.is_multicast() {
                return false;
            }
            // Unspecified - IETF RFC 5771
            if ipv4.is_unspecified() {
                return false;
            }
            // Global addresses are permitted
            // Loopback addresses are permitted
            // (127.0.0.0/8)
            // Private addresses are permitted
            // (10.0.0.0/8, 172.16.0.0/12 and 192.168.0.0/16)
            //
            return true;
        }
        Err(_) => return false
    }
    return false;
}
