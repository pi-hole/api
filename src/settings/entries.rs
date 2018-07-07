// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Entry specifications for SetupVars & FTL Configuration Files
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use settings::value_type::ValueType;
use std::borrow::Cow;

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
    pub fn key(&self) -> Cow<'static, str> {
        match *self {
            SetupVarsEntry::ApiQueryLogShow => Cow::Borrowed("API_QUERY_LOG_SHOW"),
            SetupVarsEntry::ApiPrivacyMode => Cow::Borrowed("API_PRIVACY_MODE"),
            SetupVarsEntry::DnsBogusPriv => Cow::Borrowed("DNS_BOGUS_PRIV"),
            SetupVarsEntry::DnsFqdnRequired => Cow::Borrowed("DNS_FQDN_REQUIRED"),
            SetupVarsEntry::ConditionalForwarding => Cow::Borrowed("CONDITIONAL_FORWARDING"),
            SetupVarsEntry::ConditionalForwardingDomain => {
                Cow::Borrowed("CONDITIONAL_FORWARDING_DOMAIN")
            }
            SetupVarsEntry::ConditionalForwardingIp => Cow::Borrowed("CONDITIONAL_FORWARDING_IP"),
            SetupVarsEntry::ConditionalForwardingReverse => {
                Cow::Borrowed("CONDITIONAL_FORWARDING_REVERSE")
            }
            SetupVarsEntry::DhcpActive => Cow::Borrowed("DHCP_ACTIVE"),
            SetupVarsEntry::DhcpEnd => Cow::Borrowed("DHCP_END"),
            SetupVarsEntry::DhcpIpv6 => Cow::Borrowed("DHCP_IPv6"),
            SetupVarsEntry::DhcpLeasetime => Cow::Borrowed("DHCP_LEASETIME"),
            SetupVarsEntry::DhcpStart => Cow::Borrowed("DHCP_START"),
            SetupVarsEntry::DhcpRouter => Cow::Borrowed("DHCP_ROUTER"),
            SetupVarsEntry::DnsmasqListening => Cow::Borrowed("DNSMASQ_LISTENING"),
            SetupVarsEntry::Dnssec => Cow::Borrowed("DNSSEC"),
            SetupVarsEntry::InstallWebServer => Cow::Borrowed("INSTALL_WEB_SERVER"),
            SetupVarsEntry::InstallWebInterface => Cow::Borrowed("INSTALL_WEB_INTERFACE"),
            SetupVarsEntry::Ipv4Address => Cow::Borrowed("IPV4_ADDRESS"),
            SetupVarsEntry::Ipv6Address => Cow::Borrowed("IPV6_ADDRESS"),
            SetupVarsEntry::PiholeDns(num) => Cow::Owned(format!("PIHOLE_DNS_{}", num)),
            SetupVarsEntry::PiholeDomain => Cow::Borrowed("PIHOLE_DOMAIN"),
            SetupVarsEntry::PiholeInterface => Cow::Borrowed("PIHOLE_INTERFACE"),
            SetupVarsEntry::QueryLogging => Cow::Borrowed("QUERY_LOGGING"),
            SetupVarsEntry::WebEnabled => Cow::Borrowed("WEB_ENABLED"),
            SetupVarsEntry::WebPassword => Cow::Borrowed("WEBPASSWORD"),
            SetupVarsEntry::WebUiBoxedLayout => Cow::Borrowed("WEBUIBOXEDLAYOUT")
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

#[cfg(test)]
mod tests {
    use super::{FTLConfEntry, SetupVarsEntry};

    #[test]
    fn test_validate_setupvars_valid() {
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
            assert_eq!(setting.is_valid(value), result);
        }
    }

    #[test]
    fn test_validate_setupvars_invalid() {
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
            assert_eq!(setting.is_valid(value), result);
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
            (FTLConfEntry::DbFile, "/etc/pihole/FTL.conf", true),
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
            assert_eq!(setting.is_valid(value), result);
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
            assert_eq!(setting.is_valid(value), result);
        }
    }
}
