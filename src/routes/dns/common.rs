// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Functions For DNS Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    routes::dns::list::List,
    util::{Error, ErrorKind}
};
use failure::ResultExt;
use regex::Regex;
use std::process::{Command, Stdio};

/// Check if a domain is valid
pub fn is_valid_domain(domain: &str) -> bool {
    let valid_chars_regex = Regex::new(
        "^((-|_)*[a-z0-9]((-|_)*[a-z0-9])*(-|_)*)(\\.(-|_)*([a-z0-9]((-|_)*[a-z0-9])*))*$"
    )
    .unwrap();
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

/// Reload Gravity to activate changes in lists
pub fn reload_gravity(list: List, env: &Env) -> Result<(), Error> {
    // Don't actually reload Gravity during testing
    if env.is_test() {
        return Ok(());
    }

    let status = Command::new("sudo")
        .arg("pihole")
        .arg("-g")
        .arg("--skip-download")
        // Based on what list we modified, only reload what is necessary
        .arg(match list {
            List::White => "--whitelist-only",
            List::Black => "--blacklist-only",
            _ => return Err(Error::from(ErrorKind::Unknown))
        })
        // Ignore stdin, stdout, and stderr
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Get the returned status code
        .status()
        .context(ErrorKind::GravityError)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::from(ErrorKind::GravityError))
    }
}
