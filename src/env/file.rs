// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Pi-hole Files
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

/// Pi-hole files used by the API
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum PiholeFile {
    DnsmasqConfig,
    Whitelist,
    Blacklist,
    Regexlist,
    SetupVars,
    FtlConfig,
    LocalVersions,
    LocalBranches
}

impl PiholeFile {
    /// Get the default location of the file
    pub fn default_location(&self) -> &'static str {
        match *self {
            PiholeFile::DnsmasqConfig => "/etc/dnsmasq.d/pihole.conf",
            PiholeFile::Whitelist => "/etc/pihole/whitelist.txt",
            PiholeFile::Blacklist => "/etc/pihole/blacklist.txt",
            PiholeFile::Regexlist => "/etc/pihole/regex.list",
            PiholeFile::SetupVars => "/etc/pihole/setupVars.conf",
            PiholeFile::FtlConfig => "/etc/pihole/pihole-FTL.conf",
            PiholeFile::LocalVersions => "/etc/pihole/localversions",
            PiholeFile::LocalBranches => "/etc/pihole/localbranches"
        }
    }
}
