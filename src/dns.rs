/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  DNS API Endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use rocket_contrib::Json;
use regex::Regex;

use util;

/// Represents one of the lists of domains used by Gravity
#[derive(PartialEq)]
enum List {
    Whitelist,
    Blacklist,
    Wildlist
}

impl List {
    /// The location of the list in the filesystem
    fn location(&self) -> &str {
        match *self {
            List::Whitelist => "/etc/pihole/whitelist.txt",
            List::Blacklist => "/etc/pihole/blacklist.txt",
            List::Wildlist => "/etc/dnsmasq.d/03-pihole-wildcard.conf"
        }
    }
}

/// Represents an API input containing a domain
#[derive(Deserialize)]
pub struct DomainInput {
    domain: String
}

/// Check if a domain is valid
fn is_valid_domain(domain: &str) -> bool {
    let valid_chars_regex = Regex::new("^((-|_)*[a-z0-9]((-|_)*[a-z0-9])*(-|_)*)(\\.(-|_)*([a-z0-9]((-|_)*[a-z0-9])*))*$").unwrap();
    let total_length_regex = Regex::new("^.{1,253}$").unwrap();
    let label_length_regex = Regex::new("^[^\\.]{1,63}(\\.[^\\.]{1,63})*$").unwrap();

    valid_chars_regex.is_match(domain)
        && total_length_regex.is_match(domain)
        && label_length_regex.is_match(domain)
}

/// Read in a value from setupVars.conf
fn read_setup_vars(entry: &str) -> io::Result<Option<String>> {
    // Open setupVars.conf
    let file = File::open("/etc/pihole/setupVars.conf")?;
    let reader = BufReader::new(file);

    // Check every line for the key (filter out lines which could not be read)
    for line in reader.lines().filter_map(|item| item.ok()) {
        // Check if we found the key
        // TODO: check if the key is on the left before trying to return
        if line.contains(entry) {
            return Ok(
                // Get the right hand side if it exists and is not empty
                line.split("=")
                    .nth(1)
                    .and_then(|item| if item.len() == 0 { None } else { Some(item) })
                    .map(|item| item.to_owned())
            )
        }
    }

    Ok(None)
}

/// Read in the domains from a list
fn get_list(list: List) -> util::Reply {
    let file = match File::open(list.location()) {
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
        let ipv4 = read_setup_vars("IPV4_ADDRESS")?;
        let ipv6 = read_setup_vars("IPV6_ADDRESS")?;

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

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist() -> util::Reply {
    get_list(List::Whitelist)
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist() -> util::Reply {
    get_list(List::Blacklist)
}

/// Get the Wildcard list domains
#[get("/dns/wildlist")]
pub fn get_wildlist() -> util::Reply {
    get_list(List::Wildlist)
}

/// Get the DNS blocking status
#[get("/dns/status")]
pub fn status() -> util::Reply {
    let file = File::open("/etc/dnsmasq.d/01-pihole.conf");

    let status = if file.is_err() {
        // If we failed to open the file, then the status is unknown
        "unknown"
    } else {
        // Read the file to a buffer
        let mut buffer = String::new();
        file?.read_to_string(&mut buffer)?;

        // Check if the gravity.list line is disabled
        let disabled = buffer.lines()
            .filter(|line| *line == "#addn-hosts=/etc/pihole/gravity.list")
            .count();

        // Get the status string
        match disabled {
            0 => "enabled",
            1 => "disabled",
            _ => "unknown"
        }
    };

    util::reply_data(json!({
        "status": status
    }))
}

/// Reload Gravity to activate changes in lists
fn reload_gravity(list: List) -> Result<(), util::Error> {
    let status = Command::new("sudo")
        .arg("pihole")
        .arg("-g")
        .arg("--skip-download")
        // Based on what list we modified, only reload what is necessary
        .arg(match list {
            List::Whitelist => "--whitelist-only",
            List::Blacklist => "--blacklist-only",
            List::Wildlist => "--wildcard-only"
        })
        // Ignore stdin, stdout, and stderr
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Get the returned status code
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(util::Error::GravityError)
    }
}

/// Add a domain to a list
fn add_list(list: List, domain: &str) -> Result<(), util::Error> {
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

/// Add a domain to the whitelist
#[post("/dns/whitelist", data = "<domain_input>")]
pub fn add_whitelist(domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the whitelist and remove it from the other lists
    add_list(List::Whitelist, domain)?;
    try_remove_list(List::Blacklist, domain)?;
    try_remove_list(List::Wildlist, domain)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    reload_gravity(List::Whitelist)?;
    util::reply_success()
}

/// Add a domain to the blacklist
#[post("/dns/blacklist", data = "<domain_input>")]
pub fn add_blacklist(domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We need to add it to the blacklist and remove it from the other lists
    add_list(List::Blacklist, domain)?;
    try_remove_list(List::Whitelist, domain)?;
    try_remove_list(List::Wildlist, domain)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    reload_gravity(List::Blacklist)?;
    util::reply_success()
}

/// Add a domain to the wildcard list
#[post("/dns/wildlist", data = "<domain_input>")]
pub fn add_wildlist(domain_input: Json<DomainInput>) -> util::Reply {
    let domain = &domain_input.0.domain;

    // We only need to add it to the wildcard list (this is the same functionality as list.sh)
    add_list(List::Wildlist, domain)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    reload_gravity(List::Wildlist)?;
    util::reply_success()
}

/// Try to remove a domain from the list, but it is not an error if the domain does not exist there
fn try_remove_list(list: List, domain: &str) -> Result<(), util::Error> {
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
fn remove_list(list: List, domain: &str) -> Result<(), util::Error> {
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

/// Delete a domain from the whitelist
#[delete("/dns/whitelist/<domain>")]
pub fn delete_whitelist(domain: String) -> util::Reply {
    remove_list(List::Whitelist, &domain)?;
    reload_gravity(List::Whitelist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the blacklist
#[delete("/dns/blacklist/<domain>")]
pub fn delete_blacklist(domain: String) -> util::Reply {
    remove_list(List::Blacklist, &domain)?;
    reload_gravity(List::Blacklist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}

/// Delete a domain from the wildcard list
#[delete("/dns/wildlist/<domain>")]
pub fn delete_wildlist(domain: String) -> util::Reply {
    remove_list(List::Wildlist, &domain)?;
    reload_gravity(List::Wildlist)?;

    // At this point, since we haven't hit an error yet, reload gravity and return success
    util::reply_success()
}
