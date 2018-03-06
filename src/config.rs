use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

/// Some of the files exposed by the `Config`
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum PiholeFile {
    DnsmasqMainConfig,
    Whitelist,
    Blacklist,
    Wildlist,
    SetupVars
}

impl PiholeFile {
    fn default_location(&self) -> &'static str {
        match *self {
            PiholeFile::DnsmasqMainConfig => "/etc/dnsmasq.d/01-pihole.conf",
            PiholeFile::Whitelist => "/etc/pihole/whitelist.txt",
            PiholeFile::Blacklist => "/etc/pihole/blacklist.txt",
            PiholeFile::Wildlist => "/etc/dnsmasq.d/03-pihole-wildcard.conf",
            PiholeFile::SetupVars => "/etc/pihole/setupVars.conf"
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

    pub fn write_file(
        &self,
        file: PiholeFile,
        open_options: OpenOptions
    ) -> io::Result<File> {
        match *self {
            Config::Production => {
                open_options.open(self.file_location(file))
            },
            Config::Test(ref map) => {
                match map.get(&file) {
                    Some(data) => data,
                    None => return Err(io::Error::new(io::ErrorKind::NotFound, "Missing test data"))
                }.try_clone()
            }
        }
    }

    /// Check if a file exists
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