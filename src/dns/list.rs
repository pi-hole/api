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
use std::io::{BufReader, BufWriter};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use util;
use dns::common::is_valid_domain;

/// Represents one of the lists of domains used by Gravity
#[derive(PartialEq)]
pub enum List {
    Whitelist,
    Blacklist,
    Wildlist
}

impl List {
    /// The location of the list in the filesystem
    pub fn location(&self) -> &str {
        match *self {
            List::Whitelist => "/etc/pihole/whitelist.txt",
            List::Blacklist => "/etc/pihole/blacklist.txt",
            List::Wildlist => "/etc/dnsmasq.d/03-pihole-wildcard.conf"
        }
    }
}

/// Add a domain to a list
pub fn add_list(list: List, domain: &str) -> Result<(), util::Error> {
    // Check if it's a valid domain before doing anything
    if !is_valid_domain(domain) {
        return Err(util::Error::InvalidDomain);
    }

    let list_path = Path::new(list.location());
    let mut domains = Vec::new();

    // Read list domains (if the list exists, otherwise the list is empty)
    if list_path.is_file() {
        let reader = BufReader::new(File::open(list_path)?);

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

    // Open the list file (and create it if it doesn't exist)
    let mut list_file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o644)
        .open(list_path)?;

    // Add the domain to the end of the list
    writeln!(list_file, "{}", domain)?;

    Ok(())
}

/// Try to remove a domain from the list, but it is not an error if the domain does not exist there
pub fn try_remove_list(list: List, domain: &str) -> Result<(), util::Error> {
    match remove_list(list, domain) {
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
pub fn remove_list(list: List, domain: &str) -> Result<(), util::Error> {
    // Check if it's a valid domain before doing anything
    if !is_valid_domain(domain) {
        return Err(util::Error::InvalidDomain);
    }

    let list_path = Path::new(list.location());
    let mut domains = Vec::new();

    // Read list domains (if the list exists, otherwise the list is empty)
    if list_path.is_file() {
        let reader = BufReader::new(File::open(list_path)?);

        // Add domains
        domains.extend(reader
            .lines()
            .filter_map(|line| line.ok())
            // Only get valid domains
            .filter(|domain| is_valid_domain(domain))
        );
    }

    // Check if the domain is already in the list
    if !domains.contains(&domain.to_owned()) {
        return Err(util::Error::NotFound);
    }

    // Open the list file (and create it if it doesn't exist). This will truncate the list so we can
    // add all the domains except the one we are deleting
    let list_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o644)
        .open(list_path)?;

    let mut writer = BufWriter::new(list_file);

    // Write all domains except the one we're deleting
    for domain in domains.into_iter().filter(|item| item != domain) {
        writer.write_all(domain.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}
