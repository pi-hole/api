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
pub enum FtlConfEntry {
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

impl FtlConfEntry {
    /// Get the pihole-FTL.conf key
    pub fn key(&self) -> &'static str {
        match *self {
            FtlConfEntry::SocketListening => "SOCKET_LISTENING",
            FtlConfEntry::QueryDisplay => "QUERY_DISPLAY",
            FtlConfEntry::AaaaQueryAnalysis => "AAAA_QUERY_ANALYSIS",
            FtlConfEntry::ResolveIpv6 => "RESOLVE_IPV6",
            FtlConfEntry::ResolveIpv4 => "RESOLVE_IPV6",
            FtlConfEntry::MaxDbDays => "MAXDBDAYS",
            FtlConfEntry::DbInterval => "DBINTERVAL",
            FtlConfEntry::DbFile => "DBFILE",
            FtlConfEntry::MaxLogAge => "MAXLOGAGE",
            FtlConfEntry::FtlPort => "FTLPORT",
            FtlConfEntry::PrivacyLevel => "PRIVACYLEVEL",
            FtlConfEntry::IgnoreLocalHost => "IGNORE_LOCALHOST",
            FtlConfEntry::BlockingMode => "BLOCKINGMODE"
        }
    }

    /// Get the acceptable value type
    pub fn value_type(&self) -> ValueType {
        match *self {
            FtlConfEntry::SocketListening => ValueType::String(&["localonly", "all"]),
            FtlConfEntry::QueryDisplay => ValueType::YesNo,
            FtlConfEntry::AaaaQueryAnalysis => ValueType::YesNo,
            FtlConfEntry::ResolveIpv6 => ValueType::YesNo,
            FtlConfEntry::ResolveIpv4 => ValueType::YesNo,
            FtlConfEntry::MaxDbDays => ValueType::Integer,
            FtlConfEntry::DbInterval => ValueType::Decimal,
            FtlConfEntry::DbFile => ValueType::Path,
            FtlConfEntry::MaxLogAge => ValueType::Decimal,
            FtlConfEntry::FtlPort => ValueType::PortNumber,
            FtlConfEntry::PrivacyLevel => ValueType::String(&["0", "1", "2", "3"]),
            FtlConfEntry::IgnoreLocalHost => ValueType::YesNo,
            FtlConfEntry::BlockingMode => {
                ValueType::String(&["NULL", "IP-AAAA-NODATA", "IP", "NXDOMAIN"])
            }
        }
    }

    /// Check if the value is valid for this entry
    pub fn is_valid(&self, value: &str) -> bool {
        self.value_type().is_valid(value)
    }
}
