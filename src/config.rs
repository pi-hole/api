use std::fs::File;
use std::io::{self, BufReader, Cursor};
use std::io::prelude::*;
use std::collections::HashMap;

/// Some of the files exposed by the `Config`
#[derive(Eq, PartialEq, Hash)]
pub enum PiholeFile {
    DnsmasqMainConfig
}

impl PiholeFile {
    fn default_location(&self) -> &'static str {
        match *self {
            PiholeFile::DnsmasqMainConfig => "/etc/dnsmasq.d/01-pihole.conf"
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
    pub fn read_file<'a>(&'a self, file: PiholeFile) -> io::Result<Box<Read + 'a>> {
        match *self {
            Config::Production => {
                let file = File::open(self.file_location(file))?;
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
}