// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// List Structure And Operations For DNS Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::{Env, PiholeFile},
    routes::dns::common::{is_valid_domain, is_valid_regex},
    util::{Error, ErrorKind}
};
use failure::ResultExt;
use std::io::{prelude::*, BufReader, BufWriter};

pub enum List {
    White,
    Black,
    Regex
}

impl List {
    /// Get the associated `PiholeFile`
    fn file(&self) -> PiholeFile {
        match *self {
            List::White => PiholeFile::Whitelist,
            List::Black => PiholeFile::Blacklist,
            List::Regex => PiholeFile::Regexlist
        }
    }

    /// Check if the list accepts the domain as valid
    fn accepts(&self, domain: &str) -> bool {
        match *self {
            List::Regex => is_valid_regex(domain),
            _ => is_valid_domain(domain)
        }
    }

    /// Read in the domains from the list
    pub fn get(&self, env: &Env) -> Result<Vec<String>, Error> {
        let file = match env.read_file(self.file()) {
            Ok(f) => f,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    // If the file is not found, then the list is empty
                    return Ok(Vec::new());
                } else {
                    return Err(e);
                }
            }
        };

        Ok(BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty())
            .collect())
    }

    /// Add a domain to the list
    pub fn add(&self, domain: &str, env: &Env) -> Result<(), Error> {
        // Check if it's a valid domain before doing anything
        if !self.accepts(domain) {
            return Err(Error::from(ErrorKind::InvalidDomain));
        }

        // Check if the domain is already in the list
        if self.get(env)?.contains(&domain.to_owned()) {
            return Err(Error::from(ErrorKind::AlreadyExists));
        }

        // Open the list file in append mode (and create it if it doesn't exist)
        let mut file = env.write_file(self.file(), true)?;

        // Add the domain to the list
        writeln!(file, "{}", domain).context(ErrorKind::FileWrite(
            env.file_location(self.file()).to_owned()
        ))?;

        Ok(())
    }

    /// Try to remove a domain from the list, but it is not an error if the
    /// domain does not exist
    pub fn try_remove(&self, domain: &str, env: &Env) -> Result<(), Error> {
        match self.remove(domain, env) {
            // Pass through successful results
            Ok(_) => Ok(()),
            Err(e) => {
                // Ignore NotFound errors
                if e.kind() == ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Remove a domain from the list
    pub fn remove(&self, domain: &str, env: &Env) -> Result<(), Error> {
        // Check if it's a valid domain before doing anything
        if !self.accepts(domain) {
            return Err(Error::from(ErrorKind::InvalidDomain));
        }

        // Check if the domain is not in the list
        let domains = self.get(env)?;
        if !domains.contains(&domain.to_owned()) {
            return Err(Error::from(ErrorKind::NotFound));
        }

        // Open the list file (and create it if it doesn't exist). This will truncate
        // the list so we can add all the domains except the one we are deleting
        let file = env.write_file(self.file(), false)?;
        let mut writer = BufWriter::new(file);

        // Write all domains except the one we're deleting
        for domain in domains.into_iter().filter(|item| item != domain) {
            writeln!(writer, "{}", domain).context(ErrorKind::FileWrite(
                env.file_location(self.file()).to_owned()
            ))?;
        }

        Ok(())
    }
}
