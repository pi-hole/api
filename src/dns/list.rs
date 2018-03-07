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
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

use util;
use dns::common::{is_valid_domain, read_setup_vars};
use config::{Config, PiholeFile};

/// Represents one of the lists of domains used by Gravity
#[derive(PartialEq)]
pub enum List {
    Whitelist,
    Blacklist,
    Wildlist
}

impl List {
    fn get_file(&self) -> PiholeFile {
        match *self {
            List::Whitelist => PiholeFile::Whitelist,
            List::Blacklist => PiholeFile::Blacklist,
            List::Wildlist => PiholeFile::Wildlist
        }
    }
}

/// Read in the domains from a list
pub fn get_list(list: List, config: &Config) -> util::Reply {
    let file = match config.read_file(list.get_file()) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                // If the file is not found, then the list is empty. [0; 0] is an empty list of
                // type i32. We can't use [] because the type needs to be known.
                return util::reply_data([0; 0]);
            } else {
                return Err(e.into());
            }
        }
    };
    let reader = BufReader::new(file);

    // Used for the wildcard list to skip IPv6 lines
    let mut skip_lines = false;
    let is_wildcard = list == List::Wildlist;

    if is_wildcard {
        // Check if both IPv4 and IPv6 are used.
        // If so, skip every other line if we're getting wildcard domains.
        let ipv4 = read_setup_vars("IPV4_ADDRESS", config)?;
        let ipv6 = read_setup_vars("IPV6_ADDRESS", config)?;

        skip_lines = ipv4.is_some() && ipv6.is_some();
    }

    let mut skip = true;
    let mut lines = Vec::new();

    // Read in the domains
    for line in reader.lines().filter_map(|item| item.ok()) {
        // Skip empty lines
        if line.len() == 0 {
            continue;
        }

        // The wildcard list sometimes skips every other, see above
        if skip_lines {
            skip = !skip;

            if skip {
                continue;
            }
        }

        // Parse the line
        let mut parsed_line = if is_wildcard {
            // If we're reading wildcards, the domain is between two forward slashes
            match line.split("/").nth(1) {
                Some(s) => s.to_owned(),
                None => continue
            }
        } else {
            line
        };

        lines.push(parsed_line);
    }

    util::reply_data(lines)
}

/// Add a domain to a list
pub fn add_list(list: List, domain: &str, config: &Config) -> Result<(), util::Error> {
    // Check if it's a valid domain before doing anything
    if !is_valid_domain(domain) {
        return Err(util::Error::InvalidDomain);
    }

    let list_file = list.get_file();
    let mut domains = Vec::new();

    // Read list domains (if the list exists, otherwise the list is empty)
    if config.file_exists(list_file) {
        let reader = BufReader::new(config.read_file(list_file)?);

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
    let mut file_options = OpenOptions::new();
    file_options
        .create(true)
        .append(true)
        .mode(0o644);

    let mut list_file = config.write_file(list_file, file_options)?;

    // Add the domain to the list (account for wildlist format)
    if list == List::Wildlist {
        if let Some(ipv4) = read_setup_vars("IPV4_ADDRESS", config)? {
            writeln!(list_file, "address=/{}/{}", domain, ipv4)?;
        }

        if let Some(ipv6) = read_setup_vars("IPV6_ADDRESS", config)? {
            writeln!(list_file, "address=/{}/{}", domain, ipv6)?;
        }
    } else {
        writeln!(list_file, "{}", domain)?;
    }

    Ok(())
}

/// Try to remove a domain from the list, but it is not an error if the domain does not exist there
pub fn try_remove_list(list: List, domain: &str, config: &Config) -> Result<(), util::Error> {
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
pub fn remove_list(list: List, domain: &str, config: &Config) -> Result<(), util::Error> {
    // Check if it's a valid domain before doing anything
    if !is_valid_domain(domain) {
        return Err(util::Error::InvalidDomain);
    }

    let list_file = list.get_file();
    let mut domains = Vec::new();

    // Read list domains (if the list exists, otherwise the list is empty)
    if config.file_exists(list_file) {
        let reader = BufReader::new(config.read_file(list_file)?);

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
    let mut file_options = OpenOptions::new();
    file_options
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o644);

    let list_file = config.write_file(list_file, file_options)?;
    let mut writer = BufWriter::new(list_file);

    // Write all domains except the one we're deleting
    for domain in domains.into_iter().filter(|item| item != domain) {
        writer.write_all(domain.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}
