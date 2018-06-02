/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Common functions for DNS endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Config, PiholeFile};
use regex::Regex;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::process::{Command, Stdio};
use util;

/// Check if a domain is valid
pub fn is_valid_domain(domain: &str) -> bool {
    let valid_chars_regex = Regex::new("^((-|_)*[a-z0-9]((-|_)*[a-z0-9])*(-|_)*)(\\.(-|_)*([a-z0-9]((-|_)*[a-z0-9])*))*$").unwrap();
    let total_length_regex = Regex::new("^.{1,253}$").unwrap();
    let label_length_regex = Regex::new("^[^\\.]{1,63}(\\.[^\\.]{1,63})*$").unwrap();

    valid_chars_regex.is_match(domain)
        && total_length_regex.is_match(domain)
        && label_length_regex.is_match(domain)
}

/// Check if a regex is valid
pub fn is_valid_regex(regex_str: &str) -> bool {
    Regex::new(regex_str).is_ok()
}

/// Read in a value from setupVars.conf
#[allow(unused)]
pub fn read_setup_vars(entry: &str, config: &Config) -> io::Result<Option<String>> {
    // Open setupVars.conf
    let reader = BufReader::new(config.read_file(PiholeFile::SetupVars)?);

    // Check every line for the key (filter out lines which could not be read)
    for line in reader.lines().filter_map(|item| item.ok()) {
        // Ignore lines without the entry as a substring
        if !line.contains(entry) {
            continue;
        }

        let mut split = line.split("=");

        // Check if we found the key by checking if the line starts with `entry=`
        if split.next().map_or(false, |section| section == entry) {
            return Ok(
                // Get the right hand side if it exists and is not empty
                split
                    .next()
                    .and_then(|item| if item.len() == 0 { None } else { Some(item) })
                    .map(|item| item.to_owned())
            )
        }
    }

    Ok(None)
}

/// Reload Gravity to activate changes in lists
pub fn reload_gravity(list: PiholeFile, config: &Config) -> Result<(), util::Error> {
    // Don't actually reload Gravity during testing
    if config.is_test() {
        return Ok(());
    }

    let status = Command::new("sudo")
        .arg("pihole")
        .arg("-g")
        .arg("--skip-download")
        // Based on what list we modified, only reload what is necessary
        .arg(match list {
            PiholeFile::Whitelist => "--whitelist-only",
            PiholeFile::Blacklist => "--blacklist-only",
            _ => return Err(util::Error::Unknown)
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
