// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Entry specifications for SetupVars & FTL Configuration Files
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::PiholeFile;
use settings::value_type::ValueType;
use std::borrow::Cow;
use util::{Error, ErrorKind};
use env::Env;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use failure::Fail;

/// Common functions for a configuration entry
pub trait ConfigEntry {
    /// Get the config file
    fn file(&self) -> PiholeFile;

    /// Get the entry key
    fn key(&self) -> Cow<str>;

    /// Get the acceptable value type
    fn value_type(&self) -> ValueType;

    /// Get the default value of the entry
    fn get_default(&self) -> &str;

    /// Check if the value is valid for this entry
    fn is_valid(&self, value: &str) -> bool {
        self.value_type().is_valid(value)
    }

    /// Read this setting from the config file it appears in.
    /// If the setting is not found, its default value is returned.
    fn read(&self, env: &Env) -> Result<String, Error> {
        let reader = BufReader::new(env.read_file(self.file())?);
        let key = self.key();

        // Check every line for the key (filter out lines which could not be read)
        for line in reader.lines().filter_map(|item| item.ok()) {
            // Ignore lines without the entry as a substring
            if !line.contains(key.as_ref()) {
                continue;
            }

            let mut split = line.split("=");

            // Check if we found the key by checking if the line starts with `entry=`
            if split.next().map_or(false, |section| section == key) {
                return Ok(
                    // Get the right hand side if it exists and is not empty
                    split.next().map_or_else(|| self.get_default().to_owned(), |item| {
                        if item.is_empty() {
                            self.get_default().to_owned()
                        } else {
                            item.to_owned()
                        }
                    })
                );
            }
        }

        Ok(self.get_default().to_owned())
    }

    /// Write a value to the config file.
    /// If the value is invalid, an error will be returned.
    fn write(&self, value: &str, env: &Env) -> Result<(), Error> {
        // Validate new value
        if !self.is_valid(value) {
            return Err(ErrorKind::InvalidSettingValue.into());
        }

        // Read specified file, removing any line matching the setting we are writing
        let key = self.key();
        let entry_equals = format!("{}=", key);
        let mut entries: Vec<String> = BufReader::new(env.read_file(self.file())?)
            .lines()
            .filter_map(|item| item.ok())
            .filter(|line| !line.starts_with(&entry_equals))
            .collect();

        // Append entry to working copy
        let new_entry = format!("{}={}", key, value);
        entries.push(new_entry);

        // Open the config file to be overwritten
        let mut file_writer = BufWriter::new(env.write_file(self.file(), false)?);

        // Create the context for the error lazily.
        // This way it is not allocating for errors at all, unless an error is thrown.
        let apply_context = |error: io::Error| {
            let context = ErrorKind::FileWrite(env.file_location(self.file()).to_owned());
            error.context(context.into())
        };

        // Write settings to file
        for line in entries {
            file_writer
                .write_all(line.as_bytes())
                .map_err(apply_context)?;
            file_writer.write_all(b"\n").map_err(apply_context)?;
        }

        file_writer.flush().map_err(apply_context)?;

        Ok(())
    }
}

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
    WebPassword,
    WebUiBoxedLayout
}

impl ConfigEntry for SetupVarsEntry {
    fn file(&self) -> PiholeFile {
        PiholeFile::SetupVars
    }

    fn key(&self) -> Cow<str> {
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
            SetupVarsEntry::WebPassword => Cow::Borrowed("WEBPASSWORD"),
            SetupVarsEntry::WebUiBoxedLayout => Cow::Borrowed("WEBUIBOXEDLAYOUT")
        }
    }

    fn value_type(&self) -> ValueType {
        match *self {
            SetupVarsEntry::ApiQueryLogShow => {
                ValueType::String(&["all", "permittedonly", "blockedonly"])
            }
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
            SetupVarsEntry::DnsmasqListening => ValueType::String(&["all", "local", "single", ""]),
            SetupVarsEntry::Dnssec => ValueType::Boolean,
            SetupVarsEntry::InstallWebServer => ValueType::Boolean,
            SetupVarsEntry::InstallWebInterface => ValueType::Boolean,
            SetupVarsEntry::Ipv4Address => ValueType::Ipv4Mask,
            SetupVarsEntry::Ipv6Address => ValueType::Ipv6,
            SetupVarsEntry::PiholeDns(_) => ValueType::Ipv4,
            SetupVarsEntry::PiholeDomain => ValueType::Domain,
            SetupVarsEntry::PiholeInterface => ValueType::Interface,
            SetupVarsEntry::QueryLogging => ValueType::Boolean,
            SetupVarsEntry::WebPassword => ValueType::WebPassword,
            SetupVarsEntry::WebUiBoxedLayout => ValueType::String(&["boxed", "traditional", ""])
        }
    }

    fn get_default(&self) -> &str {
        match *self {
            SetupVarsEntry::ApiQueryLogShow => "all",
            SetupVarsEntry::ApiPrivacyMode => "false",
            SetupVarsEntry::DnsBogusPriv => "true",
            SetupVarsEntry::DnsFqdnRequired => "true",
            SetupVarsEntry::ConditionalForwarding => "false",
            SetupVarsEntry::ConditionalForwardingDomain => "",
            SetupVarsEntry::ConditionalForwardingIp => "",
            SetupVarsEntry::ConditionalForwardingReverse => "",
            SetupVarsEntry::DhcpActive => "false",
            SetupVarsEntry::DhcpEnd => "",
            SetupVarsEntry::DhcpIpv6 => "false",
            SetupVarsEntry::DhcpLeasetime => "24",
            SetupVarsEntry::DhcpStart => "",
            SetupVarsEntry::DhcpRouter => "",
            SetupVarsEntry::DnsmasqListening => "single",
            SetupVarsEntry::Dnssec => "false",
            SetupVarsEntry::InstallWebServer => "true",
            SetupVarsEntry::InstallWebInterface => "true",
            SetupVarsEntry::Ipv4Address => "",
            SetupVarsEntry::Ipv6Address => "",
            SetupVarsEntry::PiholeDns(_) => "",
            SetupVarsEntry::PiholeDomain => "",
            SetupVarsEntry::PiholeInterface => "",
            SetupVarsEntry::QueryLogging => "true",
            SetupVarsEntry::WebPassword => "",
            SetupVarsEntry::WebUiBoxedLayout => "boxed"
        }
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

impl ConfigEntry for FtlConfEntry {
    fn file(&self) -> PiholeFile {
        PiholeFile::FtlConfig
    }

    fn key(&self) -> Cow<str> {
        Cow::Borrowed(match *self {
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
        })
    }

    fn value_type(&self) -> ValueType {
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

    fn get_default(&self) -> &str {
        match *self {
            FtlConfEntry::SocketListening => "localonly",
            FtlConfEntry::QueryDisplay => "yes",
            FtlConfEntry::AaaaQueryAnalysis => "yes",
            FtlConfEntry::ResolveIpv6 => "yes",
            FtlConfEntry::ResolveIpv4 => "yes",
            FtlConfEntry::MaxDbDays => "365",
            FtlConfEntry::DbInterval => "1.0",
            FtlConfEntry::DbFile => "/etc/pihole/pihole-FTL.db",
            FtlConfEntry::MaxLogAge => "24.0",
            FtlConfEntry::FtlPort => "4711",
            FtlConfEntry::PrivacyLevel => "0",
            FtlConfEntry::IgnoreLocalHost => "no",
            FtlConfEntry::BlockingMode => "NULL"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigEntry, SetupVarsEntry};
    use env::{Config, Env, PiholeFile};
    use testing::TestEnvBuilder;

    /// Test to make sure when writing a setting, a similar setting does not
    /// get deleted. Example: Adding PIHOLE_DNS_1 should not delete
    /// PIHOLE_DNS_10
    #[test]
    fn write_similar_keys() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_10=1.1.1.1\n",
            "PIHOLE_DNS_10=1.1.1.1\n\
             PIHOLE_DNS_1=2.2.2.2\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        SetupVarsEntry::PiholeDns(1).write("2.2.2.2", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_null_value() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=1.2.3.4\n",
            "PIHOLE_DNS_1=\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        SetupVarsEntry::PiholeDns(1).write("", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_over_duplicate_keys() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=2.2.2.2\n\
             PIHOLE_DNS_1=1.2.3.4\n",
            "PIHOLE_DNS_1=5.6.7.8\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        SetupVarsEntry::PiholeDns(1).write("5.6.7.8", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_over_null_value() {
        let env_builder = TestEnvBuilder::new().file_expect(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=\n",
            "PIHOLE_DNS_1=1.2.3.4\n"
        );
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        SetupVarsEntry::PiholeDns(1).write("1.2.3.4", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    #[test]
    fn write_to_empty_file() {
        let env_builder =
            TestEnvBuilder::new().file_expect(PiholeFile::SetupVars, "", "PIHOLE_DNS_1=1.1.1.1\n");
        let mut test_file = env_builder.get_test_files().into_iter().next().unwrap();
        let env = Env::Test(Config::default(), env_builder.build());

        SetupVarsEntry::PiholeDns(1).write("1.1.1.1", &env).unwrap();

        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }
}
