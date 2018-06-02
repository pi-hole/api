/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  List structure and operations for DNS endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};

use util;
use dns::common::{is_valid_domain, is_valid_regex};
use config::{Config, PiholeFile};

/// Check that the file is a domain list, and return `Error::Unknown` otherwise
fn verify_list(file: PiholeFile) -> Result<(), util::Error> {
    match file {
        PiholeFile::Whitelist | PiholeFile::Blacklist | PiholeFile::Regexlist => Ok(()),
        _ => Err(util::Error::Unknown)
    }
}

/// Read in the domains from a list
pub fn get_list(list: PiholeFile, config: &Config) -> Result<Vec<String>, util::Error> {
    // Only allow the domain lists to be used
    verify_list(list)?;

    let file = match config.read_file(list) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                // If the file is not found, then the list is empty
                return Ok(Vec::new());
            } else {
                return Err(e.into());
            }
        }
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| line.len() != 0)
        .collect();

    Ok(lines)
}

/// Add a domain to a list
pub fn add_list(list: PiholeFile, domain: &str, config: &Config) -> Result<(), util::Error> {
    // Only allow the domain lists to be used
    verify_list(list)?;

    // Check if it's a valid domain before doing anything
    let valid_domain = match list {
        PiholeFile::Regexlist => is_valid_regex(domain),
        _ => is_valid_domain(domain)
    };

    if !valid_domain {
        return Err(util::Error::InvalidDomain);
    }

    let mut domains = Vec::new();

    // Read list domains (if the list exists, otherwise the list is empty)
    if config.file_exists(list) {
        let reader = BufReader::new(config.read_file(list)?);

        // Add domains
        domains.extend(reader
            .lines()
            .filter_map(|line| line.ok())
            // Only get valid domains
            .filter(|domain| is_valid_domain(domain))
        );
    }

    // Check if the domain is already in the list
    if domains.contains(&domain.to_owned()) {
        return Err(util::Error::AlreadyExists);
    }

    // Open the list file in append mode (and create it if it doesn't exist)
    let mut list_file = config.write_file(list, true)?;

    // Add the domain to the list
    writeln!(list_file, "{}", domain)?;

    Ok(())
}

/// Try to remove a domain from the list, but it is not an error if the domain does not exist there
pub fn try_remove_list(list: PiholeFile, domain: &str, config: &Config) -> Result<(), util::Error> {
    match remove_list(list, domain, config) {
        // Pass through successful results
        Ok(ok) => Ok(ok),
        Err(e) => {
            // Ignore NotFound errors
            if e == util::Error::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

/// Remove a domain from a list
pub fn remove_list(list: PiholeFile, domain: &str, config: &Config) -> Result<(), util::Error> {
    // Only allow the domain lists to be used
    verify_list(list)?;

    // Check if it's a valid domain before doing anything
    let valid_domain = match list {
        PiholeFile::Regexlist => is_valid_regex(domain),
        _ => is_valid_domain(domain)
    };

    if !valid_domain {
        return Err(util::Error::InvalidDomain);
    }

    let domains = get_list(list, config)?;

    // Check if the domain is already in the list
    if !domains.contains(&domain.to_owned()) {
        return Err(util::Error::NotFound);
    }

    // Open the list file (and create it if it doesn't exist). This will truncate the list so we can
    // add all the domains except the one we are deleting
    let list_file = config.write_file(list, false)?;
    let mut writer = BufWriter::new(list_file);

    // Write all domains except the one we're deleting
    for domain in domains.into_iter().filter(|item| item != domain) {
        writeln!(writer, "{}", domain)?;
    }

    Ok(())
}
