// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Entry specifications for SetupVars & FTL Configuration Files
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::{Env, PiholeFile},
    settings::value_type::ValueType,
    util::{Error, ErrorKind}
};
use failure::{Fail, ResultExt};
use std::{
    borrow::Cow,
    io::{self, prelude::*, BufWriter},
    str::FromStr
};

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

    /// Check if the value is valid for this entry. An empty string is always
    /// valid because it represents a deleted entry.
    fn is_valid(&self, value: &str) -> bool {
        value.is_empty() || self.value_type().is_valid(value)
    }

    /// Try to read the value and parse into a boolean.
    fn is_true(&self, env: &Env) -> Result<bool, Error> {
        self.read_as::<bool>(env)
    }

    /// Try to read the value and parse into `T`.
    /// If it is unable to be parsed into `T`, an error is returned.
    fn read_as<T: FromStr>(&self, env: &Env) -> Result<T, Error>
    where
        <T as FromStr>::Err: Fail
    {
        self.read(env)?
            .parse::<T>()
            .context(ErrorKind::InvalidSettingValue)
            .map_err(Error::from)
    }

    /// Try to read the value as a comma-separated list
    fn read_list(&self, env: &Env) -> Result<Vec<String>, Error> {
        Ok(self
            .read(env)?
            .split(',')
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect())
    }

    /// Read this setting from the config file it appears in.
    /// If the setting is not found, its default value is returned.
    fn read(&self, env: &Env) -> Result<String, Error> {
        let lines = env.read_file_lines(self.file())?;
        let key = self.key();

        // Check every line for the key (filter out lines which could not be read)
        for line in lines {
            // Ignore lines without the entry as a substring
            if !line.contains(key.as_ref()) {
                continue;
            }

            let mut split = line.split('=');

            // Check if we found the key by checking if the line starts with `entry=`
            if split.next().map_or(false, |section| section == key) {
                return Ok(
                    // Get the right hand side if it exists and is not empty
                    split.next().map_or_else(
                        || self.get_default().to_owned(),
                        |item| {
                            if item.is_empty() {
                                self.get_default().to_owned()
                            } else {
                                item.to_owned()
                            }
                        }
                    )
                );
            }
        }

        Ok(self.get_default().to_owned())
    }

    /// Write a value to the config file. If the value is empty then the entry
    /// will be deleted. If the value is invalid, an error will be returned.
    fn write(&self, value: &str, env: &Env) -> Result<(), Error> {
        // Validate new value
        if !self.is_valid(value) {
            return Err(Error::from(ErrorKind::InvalidSettingValue));
        }

        // Read specified file, removing any line matching the setting we are writing
        let key = self.key();
        let entry_equals = format!("{}=", key);
        let mut entries: Vec<String> = env
            .read_file_lines(self.file())?
            .into_iter()
            .filter(|line| !line.starts_with(&entry_equals))
            .collect();

        // Append entry to working copy if not empty
        if !value.is_empty() {
            let new_entry = format!("{}={}", key, value);
            entries.push(new_entry);
        }

        // Open the config file to be overwritten
        let mut file_writer = BufWriter::new(env.write_file(self.file(), false)?);

        // Create the context for the error lazily.
        // This way it is not allocating for errors at all, unless an error is thrown.
        let apply_context = |error: io::Error| {
            error.context(ErrorKind::FileWrite(
                env.file_location(self.file()).to_owned()
            ))
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

    /// Delete the entry from the config file. This is the same as writing an
    /// empty string.
    fn delete(&self, env: &Env) -> Result<(), Error> {
        self.write("", env)
    }
}

/// setupVars.conf file entries
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum SetupVarsEntry {
    ApiExcludeClients,
    ApiExcludeDomains,
    ApiQueryLogShow,
    BlockingEnabled,
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
    DhcpRapidCommit,
    DhcpRouter,
    DnsmasqListening,
    Dnssec,
    HostRecord,
    Ipv4Address,
    Ipv6Address,
    PiholeDns(usize),
    PiholeDomain,
    PiholeInterface,
    QueryLogging,
    WebPassword,
    WebLayout,
    WebLanguage
}

impl ConfigEntry for SetupVarsEntry {
    fn file(&self) -> PiholeFile {
        PiholeFile::SetupVars
    }

    fn key(&self) -> Cow<str> {
        match self {
            SetupVarsEntry::ApiExcludeClients => Cow::Borrowed("API_EXCLUDE_CLIENTS"),
            SetupVarsEntry::ApiExcludeDomains => Cow::Borrowed("API_EXCLUDE_DOMAINS"),
            SetupVarsEntry::ApiQueryLogShow => Cow::Borrowed("API_QUERY_LOG_SHOW"),
            SetupVarsEntry::BlockingEnabled => Cow::Borrowed("BLOCKING_ENABLED"),
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
            SetupVarsEntry::DhcpRapidCommit => Cow::Borrowed("DHCP_rapid_commit"),
            SetupVarsEntry::DhcpRouter => Cow::Borrowed("DHCP_ROUTER"),
            SetupVarsEntry::DnsmasqListening => Cow::Borrowed("DNSMASQ_LISTENING"),
            SetupVarsEntry::Dnssec => Cow::Borrowed("DNSSEC"),
            SetupVarsEntry::HostRecord => Cow::Borrowed("HOSTRECORD"),
            SetupVarsEntry::Ipv4Address => Cow::Borrowed("IPV4_ADDRESS"),
            SetupVarsEntry::Ipv6Address => Cow::Borrowed("IPV6_ADDRESS"),
            SetupVarsEntry::PiholeDns(num) => Cow::Owned(format!("PIHOLE_DNS_{}", num)),
            SetupVarsEntry::PiholeDomain => Cow::Borrowed("PIHOLE_DOMAIN"),
            SetupVarsEntry::PiholeInterface => Cow::Borrowed("PIHOLE_INTERFACE"),
            SetupVarsEntry::QueryLogging => Cow::Borrowed("QUERY_LOGGING"),
            SetupVarsEntry::WebPassword => Cow::Borrowed("WEBPASSWORD"),
            SetupVarsEntry::WebLayout => Cow::Borrowed("WEBUIBOXEDLAYOUT"),
            SetupVarsEntry::WebLanguage => Cow::Borrowed("WEB_LANGUAGE")
        }
    }

    fn value_type(&self) -> ValueType {
        match self {
            SetupVarsEntry::ApiExcludeClients => {
                ValueType::Array(&[ValueType::Hostname, ValueType::IPv4, ValueType::IPv6])
            }
            SetupVarsEntry::ApiExcludeDomains => ValueType::Array(&[ValueType::Hostname]),
            SetupVarsEntry::ApiQueryLogShow => {
                ValueType::String(&["all", "permittedonly", "blockedonly", "nothing"])
            }
            SetupVarsEntry::BlockingEnabled => ValueType::Boolean,
            SetupVarsEntry::DnsBogusPriv => ValueType::Boolean,
            SetupVarsEntry::DnsFqdnRequired => ValueType::Boolean,
            SetupVarsEntry::ConditionalForwarding => ValueType::Boolean,
            SetupVarsEntry::ConditionalForwardingDomain => ValueType::Hostname,
            SetupVarsEntry::ConditionalForwardingIp => ValueType::IPv4,
            SetupVarsEntry::ConditionalForwardingReverse => ValueType::ConditionalForwardingReverse,
            SetupVarsEntry::DhcpActive => ValueType::Boolean,
            SetupVarsEntry::DhcpEnd => ValueType::IPv4,
            SetupVarsEntry::DhcpIpv6 => ValueType::Boolean,
            SetupVarsEntry::DhcpLeasetime => ValueType::Integer,
            SetupVarsEntry::DhcpStart => ValueType::IPv4,
            SetupVarsEntry::DhcpRapidCommit => ValueType::Boolean,
            SetupVarsEntry::DhcpRouter => ValueType::IPv4,
            SetupVarsEntry::DnsmasqListening => ValueType::String(&["all", "local", "single"]),
            SetupVarsEntry::Dnssec => ValueType::Boolean,
            SetupVarsEntry::HostRecord => ValueType::Domain,
            SetupVarsEntry::Ipv4Address => ValueType::IPv4Mask,
            SetupVarsEntry::Ipv6Address => ValueType::IPv6,
            SetupVarsEntry::PiholeDns(_) => {
                ValueType::Any(&[ValueType::IPv4OptionalPort, ValueType::IPv6OptionalPort])
            }
            SetupVarsEntry::PiholeDomain => ValueType::Hostname,
            SetupVarsEntry::PiholeInterface => ValueType::Interface,
            SetupVarsEntry::QueryLogging => ValueType::Boolean,
            SetupVarsEntry::WebPassword => ValueType::WebPassword,
            SetupVarsEntry::WebLayout => ValueType::String(&["boxed", "traditional"]),
            SetupVarsEntry::WebLanguage => ValueType::LanguageCode
        }
    }

    fn get_default(&self) -> &str {
        match self {
            SetupVarsEntry::ApiExcludeClients => "",
            SetupVarsEntry::ApiExcludeDomains => "",
            SetupVarsEntry::ApiQueryLogShow => "all",
            SetupVarsEntry::BlockingEnabled => "true",
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
            SetupVarsEntry::DhcpRapidCommit => "true",
            SetupVarsEntry::DhcpRouter => "",
            SetupVarsEntry::DnsmasqListening => "local",
            SetupVarsEntry::Dnssec => "false",
            SetupVarsEntry::HostRecord => "",
            SetupVarsEntry::Ipv4Address => "",
            SetupVarsEntry::Ipv6Address => "",
            SetupVarsEntry::PiholeDns(_) => "",
            SetupVarsEntry::PiholeDomain => "lan",
            SetupVarsEntry::PiholeInterface => "",
            SetupVarsEntry::QueryLogging => "false",
            SetupVarsEntry::WebPassword => "",
            SetupVarsEntry::WebLayout => "boxed",
            SetupVarsEntry::WebLanguage => "en"
        }
    }
}

impl SetupVarsEntry {
    /// Delete all `SetupVarsEntry::PiholeDns` entries
    pub fn delete_upstream_dns(env: &Env) -> Result<(), Error> {
        let entries: Vec<String> = env
            .read_file_lines(PiholeFile::SetupVars)?
            .into_iter()
            .filter(|line| !line.starts_with("PIHOLE_DNS_"))
            .collect();

        // Open the config file to be overwritten
        let mut file_writer = BufWriter::new(env.write_file(PiholeFile::SetupVars, false)?);

        // Create the context for the error lazily.
        // This way it is not allocating for errors at all, unless an error is thrown.
        let apply_context = |error: io::Error| {
            error.context(ErrorKind::FileWrite(
                env.file_location(PiholeFile::SetupVars).to_owned()
            ))
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

/// pihole-FTL.conf settings file entries
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum FtlConfEntry {
    AaaaQueryAnalysis,
    BlockingMode,
    DbFile,
    DbInterval,
    FtlPort,
    GravityDb,
    IgnoreLocalHost,
    MaxDbDays,
    MaxLogAge,
    PrivacyLevel,
    QueryDisplay,
    RegexDebugMode,
    ResolveIpv4,
    ResolveIpv6,
    SocketListening
}

impl ConfigEntry for FtlConfEntry {
    fn file(&self) -> PiholeFile {
        PiholeFile::FtlConfig
    }

    fn key(&self) -> Cow<str> {
        Cow::Borrowed(match self {
            FtlConfEntry::AaaaQueryAnalysis => "AAAA_QUERY_ANALYSIS",
            FtlConfEntry::BlockingMode => "BLOCKINGMODE",
            FtlConfEntry::DbFile => "DBFILE",
            FtlConfEntry::DbInterval => "DBINTERVAL",
            FtlConfEntry::FtlPort => "FTLPORT",
            FtlConfEntry::GravityDb => "GRAVITYDB",
            FtlConfEntry::IgnoreLocalHost => "IGNORE_LOCALHOST",
            FtlConfEntry::MaxDbDays => "MAXDBDAYS",
            FtlConfEntry::MaxLogAge => "MAXLOGAGE",
            FtlConfEntry::PrivacyLevel => "PRIVACYLEVEL",
            FtlConfEntry::QueryDisplay => "QUERY_DISPLAY",
            FtlConfEntry::RegexDebugMode => "REGEX_DEBUGMODE",
            FtlConfEntry::ResolveIpv4 => "RESOLVE_IPV6",
            FtlConfEntry::ResolveIpv6 => "RESOLVE_IPV6",
            FtlConfEntry::SocketListening => "SOCKET_LISTENING"
        })
    }

    fn value_type(&self) -> ValueType {
        match self {
            FtlConfEntry::AaaaQueryAnalysis => ValueType::YesNo,
            FtlConfEntry::BlockingMode => {
                ValueType::String(&["NULL", "IP-AAAA-NODATA", "IP", "NXDOMAIN"])
            }
            FtlConfEntry::DbFile => ValueType::Path,
            FtlConfEntry::DbInterval => ValueType::Decimal,
            FtlConfEntry::FtlPort => ValueType::PortNumber,
            FtlConfEntry::GravityDb => ValueType::Path,
            FtlConfEntry::IgnoreLocalHost => ValueType::YesNo,
            FtlConfEntry::MaxDbDays => ValueType::Integer,
            FtlConfEntry::MaxLogAge => ValueType::Decimal,
            FtlConfEntry::PrivacyLevel => ValueType::String(&["0", "1", "2", "3", "4"]),
            FtlConfEntry::QueryDisplay => ValueType::YesNo,
            FtlConfEntry::RegexDebugMode => ValueType::Boolean,
            FtlConfEntry::ResolveIpv4 => ValueType::YesNo,
            FtlConfEntry::ResolveIpv6 => ValueType::YesNo,
            FtlConfEntry::SocketListening => ValueType::String(&["localonly", "all"])
        }
    }

    fn get_default(&self) -> &str {
        match self {
            FtlConfEntry::AaaaQueryAnalysis => "yes",
            FtlConfEntry::BlockingMode => "NULL",
            FtlConfEntry::DbFile => "/etc/pihole/pihole-FTL.db",
            FtlConfEntry::DbInterval => "1.0",
            FtlConfEntry::FtlPort => "4711",
            FtlConfEntry::GravityDb => "/etc/pihole/gravity.db",
            FtlConfEntry::IgnoreLocalHost => "no",
            FtlConfEntry::MaxDbDays => "365",
            FtlConfEntry::MaxLogAge => "24.0",
            FtlConfEntry::PrivacyLevel => "0",
            FtlConfEntry::QueryDisplay => "yes",
            FtlConfEntry::RegexDebugMode => "false",
            FtlConfEntry::ResolveIpv4 => "yes",
            FtlConfEntry::ResolveIpv6 => "yes",
            FtlConfEntry::SocketListening => "localonly"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigEntry, SetupVarsEntry};
    use crate::{
        env::{Env, PiholeFile},
        testing::TestEnvBuilder
    };

    /// Run a test with a single file.
    ///
    /// The file is populated with some initial data, the test is run, and the
    /// file is checked against the expected data.
    fn test_with_file(
        file: PiholeFile,
        initial_data: &str,
        expected_data: &str,
        test_fn: impl FnOnce(Env)
    ) {
        // Create the environment
        let env_builder = TestEnvBuilder::new().file_expect(file, initial_data, expected_data);
        let mut test_file = env_builder.clone_test_files().into_iter().next().unwrap();
        let env = env_builder.build();

        // Run the test
        test_fn(env);

        // Check the file's final contents
        let mut buffer = String::new();
        test_file.assert_expected(&mut buffer);
    }

    /// Test to make sure when writing a setting, a similar setting does not
    /// get deleted. Example: Adding PIHOLE_DNS_1 should not delete
    /// PIHOLE_DNS_10
    #[test]
    fn write_similar_keys() {
        test_with_file(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_10=1.1.1.1\n",
            "PIHOLE_DNS_10=1.1.1.1\n\
             PIHOLE_DNS_1=2.2.2.2\n",
            |env| {
                SetupVarsEntry::PiholeDns(1).write("2.2.2.2", &env).unwrap();
            }
        );
    }

    /// When a entry is deleted, it is removed from the file
    #[test]
    fn delete_value() {
        test_with_file(PiholeFile::SetupVars, "PIHOLE_DNS_1=1.2.3.4\n", "", |env| {
            SetupVarsEntry::PiholeDns(1).write("", &env).unwrap();
        });
    }

    /// When an entry is written to, it cleans out any duplicates
    #[test]
    fn write_over_duplicate_keys() {
        test_with_file(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=2.2.2.2\n\
             PIHOLE_DNS_1=1.2.3.4\n",
            "PIHOLE_DNS_1=5.6.7.8\n",
            |env| {
                SetupVarsEntry::PiholeDns(1).write("5.6.7.8", &env).unwrap();
            }
        );
    }

    /// When an entry is written to, it overwrites existing values, even empty
    /// ones
    #[test]
    fn write_over_null_value() {
        test_with_file(
            PiholeFile::SetupVars,
            "PIHOLE_DNS_1=\n",
            "PIHOLE_DNS_1=1.2.3.4\n",
            |env| {
                SetupVarsEntry::PiholeDns(1).write("1.2.3.4", &env).unwrap();
            }
        );
    }

    /// Entries are written to an empty file successfully
    #[test]
    fn write_to_empty_file() {
        test_with_file(PiholeFile::SetupVars, "", "PIHOLE_DNS_1=1.1.1.1\n", |env| {
            SetupVarsEntry::PiholeDns(1).write("1.1.1.1", &env).unwrap();
        });
    }
}
