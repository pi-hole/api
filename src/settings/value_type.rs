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

#[cfg(test)]
mod tests {
    use settings::value_type::is_ipv4_valid;
    use settings::{FTLConfEntry, SetupVarsEntry};

    #[test]
    fn test_validate_setup_vars_valid() {
        let tests = vec![
            // Valid parameters
            (SetupVarsEntry::ApiQueryLogShow, "all", true),
            (SetupVarsEntry::ApiPrivacyMode, "false", true),
            (SetupVarsEntry::DnsBogusPriv, "true", true),
            (SetupVarsEntry::DnsFqdnRequired, "true", true),
            (SetupVarsEntry::ConditionalForwarding, "true", true),
            (SetupVarsEntry::ConditionalForwardingDomain, "hub", true),
            (SetupVarsEntry::ConditionalForwardingIp, "192.168.1.1", true),
            (
                SetupVarsEntry::ConditionalForwardingReverse,
                "1.168.192.in-addr.arpa",
                true
            ),
            (SetupVarsEntry::DhcpActive, "false", true),
            (SetupVarsEntry::DhcpEnd, "199.199.1.255", true),
            (SetupVarsEntry::DhcpIpv6, "false", true),
            (SetupVarsEntry::DhcpLeasetime, "24", true),
            (SetupVarsEntry::DhcpStart, "199.199.1.0", true),
            (SetupVarsEntry::DhcpRouter, "192.168.1.1", true),
            (SetupVarsEntry::DnsmasqListening, "all", true),
            (SetupVarsEntry::Dnssec, "false", true),
            (SetupVarsEntry::InstallWebServer, "true", true),
            (SetupVarsEntry::InstallWebInterface, "true", true),
            (SetupVarsEntry::Ipv4Address, "192.168.1.205/24", true),
            (
                SetupVarsEntry::Ipv6Address,
                "2001:470:66:d5f:114b:a1b9:2a13:c7d9",
                true
            ),
            (SetupVarsEntry::PiholeDns(0), "8.8.4.4", true),
            (SetupVarsEntry::PiholeDomain, "lan", true),
            (SetupVarsEntry::PiholeInterface, "lo", true),
            (SetupVarsEntry::QueryLogging, "true", true),
            (SetupVarsEntry::WebUiBoxedLayout, "boxed", true),
            (SetupVarsEntry::WebEnabled, "false", true),
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
    fn test_validate_setup_vars_invalid() {
        let tests = vec![
            // Valid parameters
            (SetupVarsEntry::ApiQueryLogShow, "41", false),
            (SetupVarsEntry::ApiPrivacyMode, "off", false),
            (SetupVarsEntry::DnsBogusPriv, "on", false),
            (SetupVarsEntry::DnsFqdnRequired, "1", false),
            (SetupVarsEntry::ConditionalForwarding, "disabled", false),
            (SetupVarsEntry::ConditionalForwardingDomain, "%%@)#", false),
            (SetupVarsEntry::ConditionalForwardingIp, "192.1.1", false),
            (
                SetupVarsEntry::ConditionalForwardingReverse,
                "in-addr.arpa.1.1.1",
                false
            ),
            (SetupVarsEntry::DhcpActive, "active", false),
            (
                SetupVarsEntry::DhcpEnd,
                "2001:470:66:d5f:114b:a1b9:2a13:c7d9",
                false
            ),
            (SetupVarsEntry::DhcpIpv6, "ipv4", false),
            (SetupVarsEntry::DhcpLeasetime, "hours", false),
            (SetupVarsEntry::DhcpStart, "199199.1.0", false),
            (SetupVarsEntry::DhcpRouter, "192.1681.1", false),
            (SetupVarsEntry::DnsmasqListening, "dnsmasq", false),
            (SetupVarsEntry::Dnssec, "1", false),
            (SetupVarsEntry::InstallWebServer, "yes", false),
            (SetupVarsEntry::InstallWebInterface, "no", false),
            (SetupVarsEntry::Ipv4Address, "192.168.1.205", false),
            (SetupVarsEntry::Ipv6Address, "192.168.1.205", false),
            (SetupVarsEntry::PiholeDns(0), "www.pi-hole.net", false),
            (SetupVarsEntry::PiholeDomain, "too many words", false),
            (SetupVarsEntry::PiholeInterface, "/dev/net/eth1", false),
            (SetupVarsEntry::QueryLogging, "disabled", false),
            (SetupVarsEntry::WebUiBoxedLayout, "true", false),
            (SetupVarsEntry::WebEnabled, "457", false),
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
    fn test_validate_setup_vars_disabled() {
        // Setting the web password is not allowed - must report false.
        assert_eq!(
            SetupVarsEntry::WebPassword
                .is_valid("B350486529B6022919491965A235157110B12437514315201184143ABBB37A14"),
            false
        );
    }

    #[test]
    fn test_validate_ftl_config_valid() {
        let tests = vec![
            // Valid values
            (FTLConfEntry::AaaaQueryAnalysis, "no", true),
            (FTLConfEntry::BlockingMode, "NULL", true),
            (FTLConfEntry::DbInterval, "5.0", true),
            (FTLConfEntry::DbFile, "/etc/test.conf", true),
            (FTLConfEntry::FtlPort, "64738", true),
            (FTLConfEntry::IgnoreLocalHost, "yes", true),
            (FTLConfEntry::MaxDbDays, "3", true),
            (FTLConfEntry::MaxLogAge, "8", true),
            (FTLConfEntry::PrivacyLevel, "2", true),
            (FTLConfEntry::QueryDisplay, "yes", true),
            (FTLConfEntry::ResolveIpv6, "yes", true),
            (FTLConfEntry::ResolveIpv4, "no", true),
            (FTLConfEntry::SocketListening, "localonly", true),
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
    fn test_validate_ftl_conf_invalid() {
        let tests = vec![
            // Invalid values
            (FTLConfEntry::AaaaQueryAnalysis, "", false),
            (FTLConfEntry::BlockingMode, "enabled", false),
            (FTLConfEntry::DbInterval, "true", false),
            (FTLConfEntry::DbFile, "FTL.conf", false),
            (FTLConfEntry::FtlPort, "65537", false),
            (FTLConfEntry::IgnoreLocalHost, "OK", false),
            (FTLConfEntry::MaxDbDays, "null", false),
            (FTLConfEntry::MaxLogAge, "enabled", false),
            (FTLConfEntry::PrivacyLevel, ">9000", false),
            (FTLConfEntry::QueryDisplay, "disabled", false),
            (FTLConfEntry::ResolveIpv6, "true", false),
            (FTLConfEntry::ResolveIpv4, "false", false),
            (FTLConfEntry::SocketListening, "eth0", false),
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
