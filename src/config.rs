/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  API Configuration
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::io;
use std::path::Path;

/// The files exposed by the `Config`
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum PiholeFile {
    DnsmasqMainConfig,
    Whitelist,
    Blacklist,
    Regexlist,
    SetupVars,
    LocalVersions,
    LocalBranches
}

impl PiholeFile {
    pub fn default_location(&self) -> &'static str {
        match *self {
            PiholeFile::DnsmasqMainConfig => "/etc/dnsmasq.d/01-pihole.conf",
            PiholeFile::Whitelist => "/etc/pihole/whitelist.txt",
            PiholeFile::Blacklist => "/etc/pihole/blacklist.txt",
            PiholeFile::Regexlist => "/etc/pihole/regex.list",
            PiholeFile::SetupVars => "/etc/pihole/setupVars.conf",
            PiholeFile::LocalVersions => "/etc/pihole/localversions",
            PiholeFile::LocalBranches => "/etc/pihole/localbranches"
        }
    }
}

/// Configuration for the Pi-hole API. Also abstracts away some systems to make testing easier
pub enum Config {
    Production, Test(HashMap<PiholeFile, File>)
}

impl Config {
    /// Get the location of a file
    pub fn file_location(&self, file: PiholeFile) -> &str {
        // TODO: read config and make a map of locations from that
        file.default_location()
    }

    /// Open a file for reading
    pub fn read_file(&self, file: PiholeFile) -> io::Result<File> {
        match *self {
            Config::Production => {
                File::open(self.file_location(file))
            },
            Config::Test(ref map) => {
                match map.get(&file) {
                    Some(data) => data,
                    None => return Err(io::Error::new(io::ErrorKind::NotFound, "Missing test data"))
                }.try_clone()
            }
        }
    }

    /// Open a file for writing. If `append` is false, the file will be truncated.
    pub fn write_file(
        &self,
        file: PiholeFile,
        append: bool
    ) -> io::Result<File> {
        match *self {
            Config::Production => {
                let mut open_options = OpenOptions::new();
                open_options
                    .create(true)
                    .write(true)
                    .mode(0o644);

                if append {
                    open_options.append(true);
                } else {
                    open_options.truncate(true);
                }

                open_options.open(self.file_location(file))
            },
            Config::Test(ref map) => {
                let file = match map.get(&file) {
                    Some(data) => data,
                    None => return Err(io::Error::new(io::ErrorKind::NotFound, "Missing test data"))
                }.try_clone()?;

                if !append {
                    file.set_len(0)?;
                }

                Ok(file)
            }
        }
    }

    /// Check if a file exists
    #[allow(unused)]
    pub fn file_exists(&self, file: PiholeFile) -> bool {
        match *self {
            Config::Production => {
                Path::new(self.file_location(file)).is_file()
            },
            Config::Test(ref map) => {
                map.contains_key(&file)
            }
        }
    }

    /// Check if we're in a testing environment
    pub fn is_test(&self) -> bool {
        match *self {
            Config::Production => false,
            Config::Test(_) => true
        }
    }
}