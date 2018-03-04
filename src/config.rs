use std::fs::File;
use std::io::{self, BufReader};
use std::io::prelude::*;

/// Some of the files exposed by the `Config`
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
    Production, Test
}

impl Config {
    /// Get the location of a file
    pub fn file_location(&self, file: PiholeFile) -> &str {
        // TODO: read config and make a map of locations from that
        file.default_location()
    }

    /// Open a file for reading
    pub fn read_file(&self, file: PiholeFile) -> io::Result<Box<Read>> {
        let file = File::open(self.file_location(file))?;
        Ok(Box::new(BufReader::new(file)))
    }
}