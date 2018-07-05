// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Entry specifications for SetupVars & FTL Configuration
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use config_files::ValueType::*;
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
    PiholeDns,
    PiholeDomain,
    PiholeInterface,
    QueryLogging,
    WebEnabled,
    WebPassword,
    WebUiBoxedLayout
}

impl SetupVarsEntry {
    /// Set the setupVars.conf key for each entry
    pub fn key(&self) -> &'static str {
        match *self {
            SetupVarsEntry::ApiQueryLogShow => "API_QUERY_LOG_SHOW",
            SetupVarsEntry::ApiPrivacyMode => "API_PRIVACY_MODE",
            SetupVarsEntry::DnsBogusPriv => "DNS_BOGUS_PRIV",
            SetupVarsEntry::DnsFqdnRequired => "DNS_FQDN_REQUIRED",
            SetupVarsEntry::ConditionalForwarding => "CONDITIONAL_FORWARDING",
            SetupVarsEntry::ConditionalForwardingDomain => "CONDITIONAL_FORWARDING_DOMAIN",
            SetupVarsEntry::ConditionalForwardingIp => "CONDITIONAL_FORWARDING_IP",
            SetupVarsEntry::ConditionalForwardingReverse => "CONDITIONAL_FORWARDING_REVERSE",
            SetupVarsEntry::DhcpActive => "DHCP_ACTIVE",
            SetupVarsEntry::DhcpEnd => "DHCP_END",
            SetupVarsEntry::DhcpIpv6 => "DHCP_IPv6",
            SetupVarsEntry::DhcpLeasetime => "DHCP_LEASETIME",
            SetupVarsEntry::DhcpStart => "DHCP_START",
            SetupVarsEntry::DhcpRouter => "DHCP_ROUTER",
            SetupVarsEntry::DnsmasqListening => "DNSMASQ_LISTENING",
            SetupVarsEntry::Dnssec => "DNSSEC",
            SetupVarsEntry::InstallWebServer => "INSTALL_WEB_SERVER",
            SetupVarsEntry::InstallWebInterface => "INSTALL_WEB_INTERFACE",
            SetupVarsEntry::Ipv4Address => "IPV4_ADDRESS",
            SetupVarsEntry::Ipv6Address => "IPV6_ADDRESS",
            SetupVarsEntry::PiholeDns => "PIHOLE_DNS_#", /* This key will need to be handled as
                                                           * a special case, replacing # as
                                                           * needed */
            SetupVarsEntry::PiholeDomain => "PIHOLE_DOMAIN",
            SetupVarsEntry::PiholeInterface => "PIHOLE_INTERFACE",
            SetupVarsEntry::QueryLogging => "QUERY_LOGGING",
            SetupVarsEntry::WebEnabled => "WEB_ENABLED",
            SetupVarsEntry::WebPassword => "WEBPASSWORD",
            SetupVarsEntry::WebUiBoxedLayout => "WEBUIBOXEDLAYOUT"
        }
    }
    /// Set the acceptable value types for each entry
    pub fn value_type(&self) -> ValueType {
        match *self {
            SetupVarsEntry::ApiQueryLogShow => ValueType::ApiQueryLogShow,
            SetupVarsEntry::ApiPrivacyMode => ValueType::Booleans,
            SetupVarsEntry::DnsBogusPriv => ValueType::Booleans,
            SetupVarsEntry::DnsFqdnRequired => ValueType::Booleans,
            SetupVarsEntry::ConditionalForwarding => ValueType::Booleans,
            SetupVarsEntry::ConditionalForwardingDomain => ValueType::Domain,
            SetupVarsEntry::ConditionalForwardingIp => ValueType::Ipv4,
            SetupVarsEntry::ConditionalForwardingReverse => ValueType::ConditionalForwardingReverse,
            SetupVarsEntry::DhcpActive => ValueType::Booleans,
            SetupVarsEntry::DhcpEnd => ValueType::Ipv4,
            SetupVarsEntry::DhcpIpv6 => ValueType::Booleans,
            SetupVarsEntry::DhcpLeasetime => ValueType::Integer,
            SetupVarsEntry::DhcpStart => ValueType::Ipv4,
            SetupVarsEntry::DhcpRouter => ValueType::Ipv4,
            SetupVarsEntry::DnsmasqListening => ValueType::DnsmasqListening,
            SetupVarsEntry::Dnssec => ValueType::Booleans,
            SetupVarsEntry::InstallWebServer => ValueType::Booleans,
            SetupVarsEntry::InstallWebInterface => ValueType::Booleans,
            SetupVarsEntry::Ipv4Address => ValueType::Ipv4Mask,
            SetupVarsEntry::Ipv6Address => ValueType::Ipv6,
            SetupVarsEntry::PiholeDns => ValueType::Ipv4,
            SetupVarsEntry::PiholeDomain => ValueType::Domain,
            SetupVarsEntry::PiholeInterface => ValueType::Interface,
            SetupVarsEntry::QueryLogging => ValueType::Booleans,
            SetupVarsEntry::WebEnabled => ValueType::Booleans,
            SetupVarsEntry::WebPassword => ValueType::WebPassword,
            SetupVarsEntry::WebUiBoxedLayout => ValueType::WebUiBoxedLayout
        }
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
            FTLConfEntry::SocketListening => ValueType::SocketListening,
            FTLConfEntry::QueryDisplay => ValueType::YesNo,
            FTLConfEntry::AaaaQueryAnalysis => ValueType::YesNo,
            FTLConfEntry::ResolveIpv6 => ValueType::YesNo,
            FTLConfEntry::ResolveIpv4 => ValueType::YesNo,
            FTLConfEntry::MaxDbDays => ValueType::Integer,
            FTLConfEntry::DbInterval => ValueType::Decimal,
            FTLConfEntry::DbFile => ValueType::Filename,
            FTLConfEntry::MaxLogAge => ValueType::Decimal,
            FTLConfEntry::FtlPort => ValueType::PortNumber,
            FTLConfEntry::PrivacyLevel => ValueType::PrivacyLevel,
            FTLConfEntry::IgnoreLocalHost => ValueType::YesNo,
            FTLConfEntry::BlockingMode => ValueType::BlockingMode
        }
    }
}

/// Categories of allowable values, shared across settings files
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueType {
    ApiQueryLogShow,
    Booleans,
    BlockingMode,
    ConditionalForwardingReverse,
    Decimal,
    DnsmasqListening,
    Domain,
    Integer,
    Interface,
    Ipv4,
    Ipv4Mask,
    Ipv6,
    Filename,
    PrivacyLevel,
    PortNumber,
    SocketListening,
    YesNo,
    WebPassword,
    WebUiBoxedLayout
}

/// Validate submitted values for each category of settings entry value
///
/// NB values are validated for format, not correctness
/// eg 0.1.2.3 is a valid IPV4, but may not be a valid upstream DNS)
///
pub fn validate_setting_value(valuetype: ValueType, value: &str) -> bool {
    match valuetype {
        ApiQueryLogShow => {
            // Specific query logging options
            match value {
                "all" | "" => true,
                _ => false
            }
        }
        Booleans => {
            // True, False or null
            match value {
                "true" | "false" | "" => true,
                _ => false
            }
        }
        BlockingMode => {
            // Specific blocking mode options
            match value {
                "NULL" | "IP-AAAA-NODATA" | "IP" | "NXDOMAIN" => true,
                _ => false
            }
        }
        ConditionalForwardingReverse => {
            // Specific reverse domain
            let reverse = Regex::new(
                r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}([a-zA-Z0-9\-\.])+$"
            ).unwrap();
            reverse.is_match(&value)
        }
        Decimal => {
            // Numberic, at least one leading digit, optional decimal point and trailing
            // digits.
            let decimal = Regex::new(r"^(\d)+(\.)?(\d)*$").unwrap();
            decimal.is_match(&value)
        }
        DnsmasqListening => {
            // Specific lisening options
            match value {
                "all" | "lan" | "single" | "" => true,
                _ => false
            }
        }
        Domain => {
            // Single word, hyphens allowed
            if value == "" {
                return true;
            };
            let domain =
                Regex::new(r"^([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])$").unwrap();
            domain.is_match(&value)
        }
        Integer => {
            // Numeric - any number of digits
            let intnum = Regex::new(r"^(\d)+$").unwrap();
            intnum.is_match(&value)
        }
        Interface => {
            // Single alphanumeric word
            let domain = Regex::new(r"^([a-zA-Z]|[a-zA-Z0-9][a-zA-Z0-9]*[a-zA-Z0-9])$").unwrap();
            domain.is_match(&value)
        }
        Ipv4 => {
            // Ipv4 4 octets, or null
            if value == "" {
                return true;
            };
            Ipv4Addr::from_str(value).is_ok()
        }
        Ipv4Mask => {
            // IPv4 - 4 octets, with mask
            let ipv4 = Regex::new(r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)/[0-9]+$").unwrap();
            ipv4.is_match(&value)
        }
        Ipv6 => {
            // IPv6 addresses, or null
            if value == "" {
                return true;
            };
            Ipv6Addr::from_str(value).is_ok()
        }
        Filename => {
            // Full path filename, or null
            if value == "" {
                return true;
            };
            let filename = Regex::new(r"^(/(\S)+)+$").unwrap();
            filename.is_match(&value)
        }
        PrivacyLevel => {
            // Specific privacy-level options
            match value {
                "0" | "1" | "2" | "3" => true,
                _ => false
            }
        }
        PortNumber => {
            // Port number, 0 - 65535
            let port = Regex::new(r"^((6553[0-5])|(655[0-2][0-9])|(65[0-4][0-9]{2})|(6[0-4][0-9]{3})|([1-5][0-9]{4})|([0-5]{0,5})|([0-9]{1,4}))$").unwrap();
            port.is_match(&value)
        }
        SocketListening => {
            // Specific socket-listening options
            match value {
                "localonly" | "all" => true,
                _ => false
            }
        }
        YesNo => {
            // Yes or no will do
            match value {
                "yes" | "no" => true,
                _ => false
            }
        }
        WebPassword => {
            // Webpassword is a valid key, but altering it is disallowed
            false
        }
        WebUiBoxedLayout => match value {
            "boxed" | "" => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use config_files::validate_setting_value;
    use config_files::FTLConfEntry::*;
    use config_files::SetupVarsEntry::*;

    #[test]
    fn test_validate_setupvars_valid() {
        let tests = [
            // Acceptable parameters
            (ApiQueryLogShow, "all", true),
            (ApiPrivacyMode, "false", true),
            (DnsBogusPriv, "true", true),
            (DnsFqdnRequired, "true", true),
            (ConditionalForwarding, "true", true),
            (ConditionalForwardingDomain, "hub", true),
            (ConditionalForwardingIp, "192.168.1.1", true),
            (ConditionalForwardingReverse, "1.168.192.in-addr.arpa", true),
            (DhcpActive, "false", true),
            (DhcpEnd, "199.199.1.255", true),
            (DhcpIpv6, "false", true),
            (DhcpLeasetime, "24", true),
            (DhcpStart, "199.199.1.0", true),
            (DhcpRouter, "192.168.1.1", true),
            (DnsmasqListening, "all", true),
            (Dnssec, "false", true),
            (InstallWebServer, "true", true),
            (InstallWebInterface, "true", true),
            (Ipv4Address, "192.168.1.205/24", true),
            (Ipv6Address, "2001:470:66:d5f:114b:a1b9:2a13:c7d9", true),
            (PiholeDns, "8.8.4.4", true),
            (PiholeDomain, "lan", true),
            (PiholeInterface, "enp0s3", true),
            (QueryLogging, "true", true),
            (WebUiBoxedLayout, "boxed", true),
            (WebEnabled, "false", true)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_setting_value(setting.value_type(), value), result);
        }
    }

    #[test]
    fn test_validate_setupvars_invalid() {
        let tests = [
            // Acceptable parameters
            (ApiQueryLogShow, "41", false),
            (ApiPrivacyMode, "off", false),
            (DnsBogusPriv, "on", false),
            (DnsFqdnRequired, "1", false),
            (ConditionalForwarding, "disabled", false),
            (ConditionalForwardingDomain, "%%@)#", false),
            (ConditionalForwardingIp, "192.1.1", false),
            (ConditionalForwardingReverse, "in-addr.arpa.1.1.1", false),
            (DhcpActive, "active", false),
            (DhcpEnd, "2001:470:66:d5f:114b:a1b9:2a13:c7d9", false),
            (DhcpIpv6, "ipv4", false),
            (DhcpLeasetime, "hours", false),
            (DhcpStart, "199199.1.0", false),
            (DhcpRouter, "192.1681.1", false),
            (DnsmasqListening, "dnsmasq", false),
            (Dnssec, "1", false),
            (InstallWebServer, "yes", false),
            (InstallWebInterface, "no", false),
            (Ipv4Address, "192.168.1.205", false),
            (Ipv6Address, "192.168.1.205", false),
            (PiholeDns, "www.pi-hole.net", false),
            (PiholeDomain, "too many words", false),
            (PiholeInterface, "/dev/net/eth1", false),
            (QueryLogging, "disabled", false),
            (WebUiBoxedLayout, "true", false),
            (WebEnabled, "457", false)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_setting_value(setting.value_type(), value), result);
        }
    }

    #[test]
    fn test_validate_setup_vars_disabled() {
        // Webpassword disallowed - must report false.
        assert_eq!(
            validate_setting_value(
                WebPassword.value_type(),
                "B350486529B6022919491965A235157110B12437514315201184143ABBB37A14"
            ),
            false
        );
    }

    #[test]
    fn test_validate_ftl_config_valid() {
        let tests = [
            // Acceptable paramaters
            (AaaaQueryAnalysis, "no", true),
            (BlockingMode, "NULL", true),
            (DbInterval, "5.0", true),
            (DbFile, "/etc/pihole/FTL.conf", true),
            (FtlPort, "64738", true),
            (IgnoreLocalHost, "yes", true),
            (MaxDbDays, "3", true),
            (MaxLogAge, "8", true),
            (PrivacyLevel, "2", true),
            (QueryDisplay, "yes", true),
            (ResolveIpv6, "yes", true),
            (ResolveIpv4, "no", true),
            (SocketListening, "localonly", true)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_setting_value(setting.value_type(), value), result);
        }
    }

    #[test]
    fn test_validate_ftl_conf_invalid() {
        let tests = [
            // Nonsensical parameters
            (AaaaQueryAnalysis, "", false),
            (BlockingMode, "enabled", false),
            (DbInterval, "true", false),
            (DbFile, "FTL.conf", false),
            (FtlPort, "65537", false),
            (IgnoreLocalHost, "OK", false),
            (MaxDbDays, "null", false),
            (MaxLogAge, "enabled", false),
            (PrivacyLevel, ">9000", false),
            (QueryDisplay, "disabled", false),
            (ResolveIpv6, "true", false),
            (ResolveIpv4, "false", false),
            (SocketListening, "eth0", false)
        ];
        for (setting, value, result) in tests.iter() {
            assert_eq!(&validate_setting_value(setting.value_type(), value), result);
        }
    }

}
