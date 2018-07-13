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
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
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
    /// Get the setupVars.conf key
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

    /// Get the acceptable value type
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

    /// Check if the value is valid for this entry
    pub fn is_valid(&self, value: &str) -> bool {
        self.value_type().is_valid(value)
    }
}

/// pihole-FTL.conf settings file entries
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
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
    /// Get the pihole-FTL.conf key
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

    /// Get the acceptable value type
    pub fn value_type(&self) -> ValueType {
        match *self {
            FTLConfEntry::SocketListening => ValueType::String(&["localonly", "all"]),
            FTLConfEntry::QueryDisplay => ValueType::YesNo,
            FTLConfEntry::AaaaQueryAnalysis => ValueType::YesNo,
            FTLConfEntry::ResolveIpv6 => ValueType::YesNo,
            FTLConfEntry::ResolveIpv4 => ValueType::YesNo,
            FTLConfEntry::MaxDbDays => ValueType::Integer,
            FTLConfEntry::DbInterval => ValueType::Decimal,
            FTLConfEntry::DbFile => ValueType::Pathname,
            FTLConfEntry::MaxLogAge => ValueType::Decimal,
            FTLConfEntry::FtlPort => ValueType::PortNumber,
            FTLConfEntry::PrivacyLevel => ValueType::String(&["0", "1", "2", "3"]),
            FTLConfEntry::IgnoreLocalHost => ValueType::YesNo,
            FTLConfEntry::BlockingMode => {
                ValueType::String(&["NULL", "IP-AAAA-NODATA", "IP", "NXDOMAIN"])
            }
        }
    }

    /// Check if the value is valid for this entry
    pub fn is_valid(&self, value: &str) -> bool {
        self.value_type().is_valid(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{FTLConfEntry, SetupVarsEntry, ValueType};

    #[test]
    fn test_ftlconf_value_types() {
        let tests = vec![
            (FTLConfEntry::AaaaQueryAnalysis, ValueType::YesNo),
            (FTLConfEntry::BlockingMode, {
                ValueType::String(&["NULL", "IP-AAAA-NODATA", "IP", "NXDOMAIN"])
            }),
            (FTLConfEntry::DbFile, ValueType::Pathname),
            (FTLConfEntry::DbInterval, ValueType::Decimal),
            (FTLConfEntry::FtlPort, ValueType::PortNumber),
            (FTLConfEntry::IgnoreLocalHost, ValueType::YesNo),
            (FTLConfEntry::MaxDbDays, ValueType::Integer),
            (FTLConfEntry::MaxLogAge, ValueType::Decimal),
            (
                FTLConfEntry::PrivacyLevel,
                ValueType::String(&["0", "1", "2", "3"])
            ),
            (FTLConfEntry::QueryDisplay, ValueType::YesNo),
            (FTLConfEntry::ResolveIpv4, ValueType::YesNo),
            (FTLConfEntry::ResolveIpv6, ValueType::YesNo),
            (
                FTLConfEntry::SocketListening,
                ValueType::String(&["localonly", "all"])
            ),
        ];

        for (entry, valuetype) in tests {
            assert_eq!(entry.value_type(), valuetype);
        }
    }

    #[test]
    fn test_setupvars_value_types() {
        let tests = vec![
            (
                SetupVarsEntry::ApiQueryLogShow,
                ValueType::String(&["all", ""])
            ),
            (SetupVarsEntry::ApiPrivacyMode, ValueType::Boolean),
            (SetupVarsEntry::DnsBogusPriv, ValueType::Boolean),
            (SetupVarsEntry::DnsFqdnRequired, ValueType::Boolean),
            (SetupVarsEntry::ConditionalForwarding, ValueType::Boolean),
            (
                SetupVarsEntry::ConditionalForwardingDomain,
                ValueType::Domain
            ),
            (SetupVarsEntry::ConditionalForwardingIp, ValueType::Ipv4),
            (
                SetupVarsEntry::ConditionalForwardingReverse,
                ValueType::ConditionalForwardingReverse
            ),
            (SetupVarsEntry::DhcpActive, ValueType::Boolean),
            (SetupVarsEntry::DhcpEnd, ValueType::Ipv4),
            (SetupVarsEntry::DhcpIpv6, ValueType::Boolean),
            (SetupVarsEntry::DhcpLeasetime, ValueType::Integer),
            (SetupVarsEntry::DhcpStart, ValueType::Ipv4),
            (SetupVarsEntry::DhcpRouter, ValueType::Ipv4),
            (
                SetupVarsEntry::DnsmasqListening,
                ValueType::String(&["all", "lan", "single", ""])
            ),
            (SetupVarsEntry::Dnssec, ValueType::Boolean),
            (SetupVarsEntry::InstallWebServer, ValueType::Boolean),
            (SetupVarsEntry::InstallWebInterface, ValueType::Boolean),
            (SetupVarsEntry::Ipv4Address, ValueType::Ipv4Mask),
            (SetupVarsEntry::Ipv6Address, ValueType::Ipv6),
            (SetupVarsEntry::PiholeDns(7), ValueType::Ipv4),
            (SetupVarsEntry::PiholeDomain, ValueType::Domain),
            (SetupVarsEntry::PiholeInterface, ValueType::Interface),
            (SetupVarsEntry::QueryLogging, ValueType::Boolean),
            (SetupVarsEntry::WebEnabled, ValueType::Boolean),
            (SetupVarsEntry::WebPassword, ValueType::WebPassword),
            (
                SetupVarsEntry::WebUiBoxedLayout,
                ValueType::String(&["boxed", ""])
            ),
        ];

        for (entry, valuetype) in tests {
            assert_eq!(entry.value_type(), valuetype);
        }
    }
}
