use std::fs::File;
use std::io::{self, BufReader, Cursor};
use std::io::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::fs::OpenOptions;

/// Some of the files exposed by the `Config`
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum PiholeFile {
    DnsmasqMainConfig,
    Whitelist,
    Blacklist,
    Wildlist
}

impl PiholeFile {
    fn default_location(&self) -> &'static str {
        match *self {
            PiholeFile::DnsmasqMainConfig => "/etc/dnsmasq.d/01-pihole.conf",
            PiholeFile::Whitelist => "/etc/pihole/whitelist.txt",
            PiholeFile::Blacklist => "/etc/pihole/blacklist.txt",
            PiholeFile::Wildlist => "/etc/dnsmasq.d/03-pihole-wildcard.conf"
        }
    }
}

/// Configuration for the Pi-hole API. Also abstracts away some systems to make testing easier
pub enum Config {
    Production, Test(HashMap<PiholeFile, Vec<u8>>)
}

impl Config {
    /// Get the location of a file
    pub fn file_location(&self, file: PiholeFile) -> &str {
        // TODO: read config and make a map of locations from that
        file.default_location()
    }

    /// Open a file for reading
    pub fn read_file<'a>(&'a self, file: PiholeFile) -> io::Result<Box<BufRead + 'a>> {
        match *self {
            Config::Production => {
                let mut file = File::open(self.file_location(file))?;

                Ok(Box::new(BufReader::new(file)))
            },
            Config::Test(ref map) => {
                let test_data = match map.get(&file) {
                    Some(data) => data,
                    None => return Err(io::Error::new(io::ErrorKind::NotFound, "Missing test data"))
                };

                Ok(Box::new(Cursor::new(test_data)))
            }
        }
    }

    pub fn write_file(
        &self,
        file: PiholeFile,
        open_options: OpenOptions
    ) -> io::Result<Box<Write>> {
        match *self {
            Config::Production => {
                let file = open_options.open(self.file_location(file))?;
                Ok(Box::new(file))
            },
            Config::Test(ref map) => {
                let test_data = match map.get(&file) {
                    Some(data) => data,
                    None => return Err(io::Error::new(io::ErrorKind::NotFound, "Missing test data"))
                }.clone();

                Ok(Box::new(Cursor::new(test_data)))
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
}