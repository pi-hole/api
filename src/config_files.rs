// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Entry specifications for SetupVars & FTL Configuration
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use regex::Regex;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// setupVars.conf file entries
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum SetupVarsEntry {
    ApiQueryLogShow,
    ApiPrivacyMode,
    DnsBogusPriv,
    DnsFqdnRequired,
    ConditionalForwarding,
    ConditionalForwardingDomain,
    ConditionalForwardingIp,
    ConditionalForwardingReverse,
    DhcpActive,
    DhcpEnd,
    DhcpIpv6,
    DhcpLeasetime,
    DhcpStart,
    DhcpRouter,
    DnsmasqListening,
    Dnssec,
    InstallWebServer,
    InstallWebInterface,
    Ipv4Address,
    Ipv6Address,
    PiholeDns(usize),
    PiholeDomain,
    PiholeInterface,
    QueryLogging,
    WebEnabled,
    WebPassword,
    WebUiBoxedLayout
}

impl SetupVarsEntry {
    /// Get the setupVars.conf key of the entry
    pub fn key(&self) -> String {
        match *self {
            SetupVarsEntry::ApiQueryLogShow => "API_QUERY_LOG_SHOW".to_owned(),
            SetupVarsEntry::ApiPrivacyMode => "API_PRIVACY_MODE".to_owned(),
            SetupVarsEntry::DnsBogusPriv => "DNS_BOGUS_PRIV".to_owned(),
            SetupVarsEntry::DnsFqdnRequired => "DNS_FQDN_REQUIRED".to_owned(),
            SetupVarsEntry::ConditionalForwarding => "CONDITIONAL_FORWARDING".to_owned(),
            SetupVarsEntry::ConditionalForwardingDomain => "CONDITIONAL_FORWARDING_DOMAIN".to_owned(),
            SetupVarsEntry::ConditionalForwardingIp => "CONDITIONAL_FORWARDING_IP".to_owned(),
            SetupVarsEntry::ConditionalForwardingReverse => "CONDITIONAL_FORWARDING_REVERSE".to_owned(),
            SetupVarsEntry::DhcpActive => "DHCP_ACTIVE".to_owned(),
            SetupVarsEntry::DhcpEnd => "DHCP_END".to_owned(),
            SetupVarsEntry::DhcpIpv6 => "DHCP_IPv6".to_owned(),
            SetupVarsEntry::DhcpLeasetime => "DHCP_LEASETIME".to_owned(),
            SetupVarsEntry::DhcpStart => "DHCP_START".to_owned(),
            SetupVarsEntry::DhcpRouter => "DHCP_ROUTER".to_owned(),
            SetupVarsEntry::DnsmasqListening => "DNSMASQ_LISTENING".to_owned(),
            SetupVarsEntry::Dnssec => "DNSSEC".to_owned(),
            SetupVarsEntry::InstallWebServer => "INSTALL_WEB_SERVER".to_owned(),
            SetupVarsEntry::InstallWebInterface => "INSTALL_WEB_INTERFACE".to_owned(),
            SetupVarsEntry::Ipv4Address => "IPV4_ADDRESS".to_owned(),
            SetupVarsEntry::Ipv6Address => "IPV6_ADDRESS".to_owned(),
            SetupVarsEntry::PiholeDns(num) => format!("PIHOLE_DNS_{}", num),
            SetupVarsEntry::PiholeDomain => "PIHOLE_DOMAIN".to_owned(),
            SetupVarsEntry::PiholeInterface => "PIHOLE_INTERFACE".to_owned(),
            SetupVarsEntry::QueryLogging => "QUERY_LOGGING".to_owned(),
            SetupVarsEntry::WebEnabled => "WEB_ENABLED".to_owned(),
            SetupVarsEntry::WebPassword => "WEBPASSWORD".to_owned(),
            SetupVarsEntry::WebUiBoxedLayout => "WEBUIBOXEDLAYOUT".to_owned()
        }
    }

    /// Set the acceptable value types for each entry
    pub fn value_type(&self) -> ValueType {
        match *self {
            SetupVarsEntry::ApiQueryLogShow => ValueType::String(&["all", ""]),
            SetupVarsEntry::ApiPrivacyMode => ValueType::Boolean,
            SetupVarsEntry::DnsBogusPriv => ValueType::Boolean,
            SetupVarsEntry::DnsFqdnRequired => ValueType::Boolean,
            SetupVarsEntry::ConditionalForwarding => ValueType::Boolean,
            SetupVarsEntry::ConditionalForwardingDomain => ValueType::Domain,
            SetupVarsEntry::ConditionalForwardingIp => ValueType::Ipv4,
            SetupVarsEntry::ConditionalForwardingReverse => ValueType::ConditionalForwardingReverse,
            SetupVarsEntry::DhcpActive => ValueType::Boolean,
            SetupVarsEntry::DhcpEnd => ValueType::Ipv4,
            SetupVarsEntry::DhcpIpv6 => ValueType::Boolean,
            SetupVarsEntry::DhcpLeasetime => ValueType::Integer,
            SetupVarsEntry::DhcpStart => ValueType::Ipv4,
            SetupVarsEntry::DhcpRouter => ValueType::Ipv4,
            SetupVarsEntry::DnsmasqListening => ValueType::String(&["all", "lan", "single", ""]),
            SetupVarsEntry::Dnssec => ValueType::Boolean,
            SetupVarsEntry::InstallWebServer => ValueType::Boolean,
            SetupVarsEntry::InstallWebInterface => ValueType::Boolean,
            SetupVarsEntry::Ipv4Address => ValueType::Ipv4Mask,
            SetupVarsEntry::Ipv6Address => ValueType::Ipv6,
            SetupVarsEntry::PiholeDns(_) => ValueType::Ipv4,
            SetupVarsEntry::PiholeDomain => ValueType::Domain,
            SetupVarsEntry::PiholeInterface => ValueType::Interface,
            SetupVarsEntry::QueryLogging => ValueType::Boolean,
            SetupVarsEntry::WebEnabled => ValueType::Boolean,
            SetupVarsEntry::WebPassword => ValueType::WebPassword,
            SetupVarsEntry::WebUiBoxedLayout => ValueType::String(&["boxed", ""])
        }
    }

    /// Validate format of supplied values
    pub fn is_valid(&self, value: &str) -> bool {
        self.value_type().is_valid(value)
    }
}

/// pihole-FTL.conf settings file entries
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum FTLConfEntry {
    AaaaQueryAnalysis,
    BlockingMode,
    ResolveIpv6,
    ResolveIpv4,
    MaxDbDays,
    DbInterval,
    DbFile,
    IgnoreLocalHost,
    FtlPort,
    MaxLogAge,
    PrivacyLevel,
    QueryDisplay,
    SocketListening
}

impl FTLConfEntry {
    /// Set the pihole-FTL.conf key strings
    pub fn key(&self) -> &'static str {
        match *self {
            FTLConfEntry::SocketListening => "SOCKET_LISTENING",
            FTLConfEntry::QueryDisplay => "QUERY_DISPLAY",
            FTLConfEntry::AaaaQueryAnalysis => "AAAA_QUERY_ANALYSIS",
            FTLConfEntry::ResolveIpv6 => "RESOLVE_IPV6",
            FTLConfEntry::ResolveIpv4 => "RESOLVE_IPV6",
            FTLConfEntry::MaxDbDays => "MAXDBDAYS",
            FTLConfEntry::DbInterval => "DBINTERVAL",
            FTLConfEntry::DbFile => "DBFILE",
            FTLConfEntry::MaxLogAge => "MAXLOGAGE",
            FTLConfEntry::FtlPort => "FTLPORT",
            FTLConfEntry::PrivacyLevel => "PRIVACYLEVEL",
            FTLConfEntry::IgnoreLocalHost => "IGNORE_LOCALHOST",
            FTLConfEntry::BlockingMode => "BLOCKINGMODE"
        }
    }

    /// Set the acceptable value types for each entry
    pub fn value_type(&self) -> ValueType {
        match *self {
            FTLConfEntry::SocketListening => ValueType::String(&["localonly", "all"]),
            FTLConfEntry::QueryDisplay => ValueType::YesNo,
            FTLConfEntry::AaaaQueryAnalysis => ValueType::YesNo,
            FTLConfEntry::ResolveIpv6 => ValueType::YesNo,
            FTLConfEntry::ResolveIpv4 => ValueType::YesNo,
            FTLConfEntry::MaxDbDays => ValueType::Integer,
            FTLConfEntry::DbInterval => ValueType::Decimal,
            FTLConfEntry::DbFile => ValueType::Filename,
            FTLConfEntry::MaxLogAge => ValueType::Decimal,
            FTLConfEntry::FtlPort => ValueType::PortNumber,
            FTLConfEntry::PrivacyLevel => ValueType::String(&["0", "1", "2", "3"]),
            FTLConfEntry::IgnoreLocalHost => ValueType::YesNo,
            FTLConfEntry::BlockingMode => {
                ValueType::String(&["NULL", "IP-AAAA-NODATA", "IP", "NXDOMAIN"])
            }
        }
    }

    /// Validate format of supplied values
    pub fn is_valid(&self, value: &str) -> bool {
        self.value_type().is_valid(value)
    }
}

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
    fn is_valid(&self, value: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use config_files::{FTLConfEntry, SetupVarsEntry};

    #[test]
    fn test_validate_setupvars_valid() {
        let tests = vec![
            // Acceptable parameters
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
            (SetupVarsEntry::PiholeInterface, "enp0s3", true),
            (SetupVarsEntry::QueryLogging, "true", true),
            (SetupVarsEntry::WebUiBoxedLayout, "boxed", true),
            (SetupVarsEntry::WebEnabled, "false", true)
        ];

        for (setting, value, result) in tests {
            assert_eq!(setting.is_valid(value), result);
        }
    }

    #[test]
    fn test_validate_setupvars_invalid() {
        let tests = vec![
            // Acceptable parameters
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
            (SetupVarsEntry::WebEnabled, "457", false)
        ];

        for (setting, value, result) in tests {
            assert_eq!(setting.is_valid(value), result);
        }
    }

    #[test]
    fn test_validate_setup_vars_disabled() {
        // Webpassword disallowed - must report false.
        assert_eq!(
            SetupVarsEntry::WebPassword
                .is_valid("B350486529B6022919491965A235157110B12437514315201184143ABBB37A14"),
            false
        );
    }

    #[test]
    fn test_validate_ftl_config_valid() {
        let tests = vec![
            // Acceptable paramaters
            (FTLConfEntry::AaaaQueryAnalysis, "no", true),
            (FTLConfEntry::BlockingMode, "NULL", true),
            (FTLConfEntry::DbInterval, "5.0", true),
            (FTLConfEntry::DbFile, "/etc/pihole/FTL.conf", true),
            (FTLConfEntry::FtlPort, "64738", true),
            (FTLConfEntry::IgnoreLocalHost, "yes", true),
            (FTLConfEntry::MaxDbDays, "3", true),
            (FTLConfEntry::MaxLogAge, "8", true),
            (FTLConfEntry::PrivacyLevel, "2", true),
            (FTLConfEntry::QueryDisplay, "yes", true),
            (FTLConfEntry::ResolveIpv6, "yes", true),
            (FTLConfEntry::ResolveIpv4, "no", true),
            (FTLConfEntry::SocketListening, "localonly", true)
        ];

        for (setting, value, result) in tests {
            assert_eq!(setting.is_valid(value), result);
        }
    }

    #[test]
    fn test_validate_ftl_conf_invalid() {
        let tests = vec![
            // Nonsensical parameters
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
            (FTLConfEntry::SocketListening, "eth0", false)
        ];

        for (setting, value, result) in tests {
            assert_eq!(setting.is_valid(value), result);
        }
    }
}
